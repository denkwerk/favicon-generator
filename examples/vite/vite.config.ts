import favicons from '@denkwerk/unplugin-favicon-generator/vite'
import { defineConfig } from 'vite'

export default defineConfig({
  plugins: [
    favicons({
      input: './assets/favicon.svg',
      themeColor: '#02969c',
      manifest: { name: 'Favicon Generator Vite Example', shortName: 'Vite Example' },
    }),
  ],
})
