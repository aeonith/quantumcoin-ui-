import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      '/address': 'http://localhost:8080',
      '/balance': 'http://localhost:8080',
      '/mine': 'http://localhost:8080',
      '/revstop': 'http://localhost:8080',
    },
  },
});