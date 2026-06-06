import react from '@vitejs/plugin-react';
import { defineConfig } from 'vitest/config';

/**
 * Vitest configuration for the Luminos control panel.
 *
 * Tests run under jsdom (required by axe-core; happy-dom is incompatible).
 * A shared setup file registers jest-dom + jest-axe matchers and resets
 * the Tauri IPC mocks between tests. Coverage uses the v8 provider.
 */
export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    // Reset `vi.fn` call history (and unstub envs) before each test so mock
    // state never leaks across tests (e.g. polling components).
    clearMocks: true,
    unstubEnvs: true,
    css: false,
    include: ['src/**/*.{test,spec}.{ts,tsx}'],
    coverage: {
      provider: 'v8',
      reporter: ['text-summary', 'text'],
      include: ['src/**/*.{ts,tsx}'],
      exclude: [
        'src/**/*.{test,spec}.{ts,tsx}',
        'src/test/**',
        'src/main.tsx',
        'src/ipc/bindings.ts',
        'src/vite-env.d.ts',
      ],
    },
  },
});
