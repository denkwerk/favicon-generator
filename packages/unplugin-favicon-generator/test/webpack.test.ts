import { readFileSync } from 'node:fs'
import { join } from 'node:path'
import { rspack } from '@rspack/core'
import webpack from 'webpack'
import { describe, expect, it } from 'vitest'
import rspackFavicons from '../src/rspack.js'
import webpackFavicons from '../src/webpack.js'
import { DEFAULT_FILES, copyFixture, listFiles } from './helpers.js'

interface Compiler {
  run: (callback: (error: Error | null, stats?: { hasErrors: () => boolean, toString: () => string }) => void) => void
}

function run(compiler: Compiler) {
  return new Promise<void>((resolve, reject) => {
    compiler.run((error, stats) => {
      if (error || stats?.hasErrors()) {
        reject(error ?? new Error(stats!.toString()))
      } else {
        resolve()
      }
    })
  })
}

const bundlers = {
  webpack: (root: string) => webpack({
    mode: 'production',
    context: root,
    entry: './main.js',
    output: { path: join(root, 'dist') },
    plugins: [webpackFavicons({ root, input: './favicon.svg', base: 'https://cdn.example.com/' })],
  }) as unknown as Compiler,
  rspack: (root: string) => rspack({
    mode: 'production',
    context: root,
    entry: './main.js',
    output: { path: join(root, 'dist') },
    plugins: [rspackFavicons({ root, input: './favicon.svg', base: 'https://cdn.example.com/' })],
  }) as unknown as Compiler,
}

for (const [name, create] of Object.entries(bundlers)) {
  describe(name, () => {
    it('emits the files and bundles the tags with base', async () => {
      const root = copyFixture('webpack')
      await run(create(root))

      const dist = join(root, 'dist')
      expect(listFiles(dist)).toEqual([...DEFAULT_FILES, 'main.js'])
      expect(readFileSync(join(dist, 'main.js'), 'utf8')).toContain('https://cdn.example.com/favicon.svg')
    })
  })
}
