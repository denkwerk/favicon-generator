// Type-level tests, checked by `tsc -p test`: defineConfig accepts valid
// configs and catches mistakes.
import { defineConfig, type FaviconConfig } from '../src/index.js'

defineConfig({ input: './logo.svg', output: './public/favicons' })
defineConfig({
  input: './logo.svg',
  themeColor: '#02969c',
  appleTouchIcon: { background: '#ffffff' },
  manifest: { name: 'App', display: 'standalone', maskable: true },
  windows: true,
  legacy: true,
  snippets: ['html', 'json'],
})
defineConfig({ appleTouchIcon: false, manifest: false, windows: { tileColor: '#000' } })
defineConfig(async (): Promise<FaviconConfig> => ({ input: './logo.svg', manifest: { display: 'minimal-ui' } }))

// @ts-expect-error typo
defineConfig({ themeColr: '#fff' })
// @ts-expect-error options of 0.1 moved into groups
defineConfig({ appName: 'App' })
// @ts-expect-error typo inside a group
defineConfig({ manifest: { nme: 'App' } })
// @ts-expect-error not a display mode
defineConfig({ manifest: { display: 'window' } })
// @ts-expect-error groups take a boolean or options
defineConfig({ windows: 'yes' })
// @ts-expect-error not a snippet
defineConfig({ snippets: ['xml'] })
