import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

/**
 * Vite configuration for the Luminos control panel.
 *
 * The dev server is fixed to port 1420 to match the Tauri `devUrl`
 * (`http://localhost:1420`) declared in `crates/luminos-app/tauri.conf.json`.
 * The production build emits to `dist/`, which Tauri consumes as
 * `frontendDist: "../ui/dist"`.
 */
export default defineConfig({
  plugins: [react()],
  // Tauri expects a fixed port; fail rather than silently pick another.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    // Target matches the webview engines Tauri ships (webkit2gtk / WebView2).
    target: 'es2022',
    outDir: 'dist',
    emptyOutDir: true,
  },
});
