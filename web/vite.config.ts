import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'

// The Rust server owns /api and /ws. In development Vite serves the client and
// forwards those paths to it; in production the server serves the built client.
const SERVER = 'http://127.0.0.1:3000'

export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    strictPort: true,
    proxy: {
      '/api': SERVER,
      '/ws': { target: SERVER, ws: true },
    },
    // Translations live in ../locales, shared with the server.
    fs: { allow: ['..'] },
  },
})
