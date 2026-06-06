import os from 'node:os';
import path from 'node:path';
import { spawn, type ChildProcess } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const dirname = fileURLToPath(new URL('.', import.meta.url));

/**
 * Absolute path to the built debug `luminos-app` binary the WebDriver session
 * drives. The worktree uses a single workspace `target/` at the repo root, so
 * from `e2e/` the binary lives one level up. The CI `test-e2e` job builds it
 * with `cargo build -p luminos-app --features tauri` before running this suite.
 */
const APPLICATION = path.resolve(dirname, '..', 'target', 'debug', 'luminos-app');

/** Path to the Rust `tauri-driver` binary (Tauri 2.x WebDriver proxy). */
const TAURI_DRIVER = path.resolve(os.homedir(), '.cargo', 'bin', 'tauri-driver');

/**
 * Headless-WebKit environment the app child needs under Xvfb (DC-10): GTK
 * window realization + software GL. Mirrors `tests/common/mod.rs` so the E2E
 * app behaves like the subprocess-harness app.
 *
 * `tauri-driver` 2.0.6's `tauri:options` supports only `application`/`args`
 * (verified against the crate source — there is NO `env` field), and the app is
 * actually launched by `WebKitWebDriver` (spawned by `tauri-driver`), inheriting
 * ITS environment. So these vars are injected into the `tauri-driver` process
 * env below, from which they propagate down the
 * `tauri-driver -> WebKitWebDriver -> app` process tree.
 */
const HEADLESS_WEBKIT_ENV: Readonly<Record<string, string>> = {
  GDK_BACKEND: 'x11',
  WEBKIT_DISABLE_COMPOSITING_MODE: '1',
  WEBKIT_DISABLE_DMABUF_RENDERER: '1',
  LIBGL_ALWAYS_SOFTWARE: '1',
};

/** The spawned `tauri-driver` child; tracked so it is reliably torn down. */
let tauriDriver: ChildProcess | undefined;
/** Set once we intend to stop the driver, so its `exit` is not treated fatal. */
let shuttingDown = false;

/** Kills the `tauri-driver` child if running. */
function closeTauriDriver(): void {
  shuttingDown = true;
  tauriDriver?.kill();
  tauriDriver = undefined;
}

/** Registers `fn` to run on every process-exit signal, then exits. */
function onShutdown(fn: () => void): void {
  const cleanup = (): void => {
    try {
      fn();
    } finally {
      process.exit();
    }
  };
  process.on('exit', fn);
  for (const signal of ['SIGINT', 'SIGTERM', 'SIGHUP'] as const) {
    process.on(signal, cleanup);
  }
}

onShutdown(closeTauriDriver);

export const config: WebdriverIO.Config = {
  runner: 'local',
  hostname: '127.0.0.1',
  port: 4444,
  // WDIO 9 loads `.ts` config + specs via `tsx` automatically (no ts-node /
  // autoCompileOpts needed).
  specs: ['./tests/**/*.e2e.ts'],
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      // The Tauri WebDriver capability: launch the built binary under wry.
      // Only `application`/`args` are supported by tauri-driver 2.0.6; the
      // headless env is injected via the `tauri-driver` process env (see
      // `beforeSession`), NOT here.
      'tauri:options': {
        application: APPLICATION,
      },
    } as WebdriverIO.Capabilities,
  ],
  logLevel: 'info',
  // Be generous: the engine seeds state, opens two windows, and the GTK/webkit
  // app is heavy under software GL. Determinism comes from `waitUntil`, not waits.
  waitforTimeout: 15_000,
  connectionRetryTimeout: 120_000,
  connectionRetryCount: 3,
  framework: 'mocha',
  reporters: ['spec'],
  mochaOpts: {
    ui: 'bdd',
    // The first session may need to start the heavy webview backend.
    timeout: 120_000,
  },

  // Start `tauri-driver` (which in turn launches the system WebKitWebDriver)
  // before the session so it can proxy the WebDriver requests on :4444.
  beforeSession: () => {
    tauriDriver = spawn(TAURI_DRIVER, [], {
      stdio: [null, process.stdout, process.stderr],
      // Inject the headless-WebKit env so it propagates down the
      // tauri-driver -> WebKitWebDriver -> app process tree (DC-10). The CI job
      // also exports these under `xvfb-run`; setting them here keeps the suite
      // robust regardless of how it is launched.
      env: { ...process.env, ...HEADLESS_WEBKIT_ENV },
    });
    tauriDriver.on('error', (error: Error) => {
      console.error('tauri-driver error:', error.message);
      process.exit(1);
    });
    tauriDriver.on('exit', (code: number | null) => {
      if (!shuttingDown) {
        console.error('tauri-driver exited unexpectedly with code:', code);
        process.exit(1);
      }
    });
  },

  // afterSession may not run if the session failed to start, so the shutdown
  // hook above is the belt-and-braces cleanup.
  afterSession: () => {
    closeTauriDriver();
  },
};
