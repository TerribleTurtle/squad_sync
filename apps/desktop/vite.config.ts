import { defineConfig, loadEnv } from 'vite';
import react from '@vitejs/plugin-react';

// https://vitejs.dev/config/
export default defineConfig(({ mode }) => {
  // Load env file based on `mode` in the current working directory.
  const env = loadEnv(mode, '.', '');

  return {
    plugins: [react()],

    // Load .env files from this directory (apps/desktop), not project root
    envDir: '.',

    // Explicitly define environment variables to ensure they're available
    define: {
      'import.meta.env.VITE_PARTYKIT_HOST': JSON.stringify(env.VITE_PARTYKIT_HOST),
    },

    // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
    //
    // 1. prevent vite from obscuring rust errors
    clearScreen: false,
    // 2. tauri expects a fixed port, fail if that port is not available
    server: {
      port: 1420,
      strictPort: true,
      watch: {
        // 3. tell vite to ignore watching `src-tauri`
        ignored: ['**/src-tauri/**'],
      },
    },
  };
});
