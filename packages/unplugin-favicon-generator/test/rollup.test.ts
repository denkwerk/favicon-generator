import { join } from 'node:path'
import { rolldown } from 'rolldown'
import { rollup } from 'rollup'
import { describe, expect, it } from 'vitest'
import rolldownFavicons from '../src/rolldown.js'
import rollupFavicons from '../src/rollup.js'
import { DEFAULT_FILES, copyFixture } from './helpers.js'

const bundlers = {
  rollup: async (root: string, pathPrefix?: string) => {
    const bundle = await rollup({ input: join(root, 'main.js'), plugins: [rollupFavicons({ root, input: './favicon.svg', pathPrefix })] })
    return (await bundle.generate({ format: 'es' })).output
  },
  rolldown: async (root: string, pathPrefix?: string) => {
    const bundle = await rolldown({ input: join(root, 'main.js'), plugins: [rolldownFavicons({ root, input: './favicon.svg', pathPrefix })] })
    return (await bundle.generate({ format: 'es' })).output
  },
}

for (const [name, bundle] of Object.entries(bundlers)) {
  describe(name, () => {
    it('emits the files as assets and bundles the tags', async () => {
      const output = await bundle(copyFixture('app'))

      const assets = output.filter((file) => file.type === 'asset').map((file) => file.fileName).sort()
      expect(assets).toEqual(DEFAULT_FILES)
      const chunk = output.find((file) => file.type === 'chunk')!
      expect(chunk.code).toContain('/apple-touch-icon.png')
    })

    it('emits below pathPrefix', async () => {
      const output = await bundle(copyFixture('app'), 'static/icons')

      const assets = output.filter((file) => file.type === 'asset').map((file) => file.fileName).sort()
      expect(assets).toEqual(DEFAULT_FILES.map((file) => `static/icons/${file}`))
      expect(output.find((file) => file.type === 'chunk')!.code).toContain('/static/icons/favicon.svg')
    })
  })
}
