import favicons from '@denkwerk/unplugin-favicon-generator/rolldown'
import { defineConfig } from 'tsdown'

export default defineConfig({
  entry: ['src/index.ts'],
  dts: true,
  plugins: [
    // tsdown builds with Rolldown, so it takes the Rolldown plugin. The files go
    // to dist/favicons/; `base` is where the consuming app serves that directory.
    favicons({
      input: './assets/favicon.svg',
      pathPrefix: '/favicons/',
      base: '/assets/my-library/',
      themeColor: '#02969c',
      manifest: { name: 'My Library', shortName: 'Library' },
    }),
  ],
})
