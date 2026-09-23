import favicons from '@denkwerk/unplugin-favicon-generator/vite'
import { defineConfig } from 'vite'

// No options here: the plugin reads favicon.config.ts.
export default defineConfig({
  plugins: [favicons()],
})
