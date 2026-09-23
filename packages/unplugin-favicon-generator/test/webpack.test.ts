import { readFileSync } from 'node:fs'
import { join } from 'node:path'
import { rspack } from '@rspack/core'
import webpack from 'webpack'
import { describe, expect, it } from 'vitest'
import rspackFavicons from '../src/rspack.js'
import webpackFavicons from '../src/webpack.js'
import { DEFAULT_FILES, copyFixture, listFiles } from './helpers.js'

interface Stats {
  hasErrors: () => boolean
  toString: () => string
}

interface Compiler {
  run: (callback: (error: Error | null, stats?: Stats) => void) => void
  close: (callback: (error?: Error | null) => void) => void
}

/** Runs the compiler once and closes it, so that no worker or handle outlives the test. */
async function run(compiler: Compiler) {
  try {
    await new Promise<void>((resolve, reject) => {
      compiler.run((error, stats) => {
        if (error || stats?.hasErrors()) {
          reject(error ?? new Error(stats!.toString()))
        } else {
          resolve()
        }
      })
    })
  } finally {
    await new Promise<void>((resolve, reject) => compiler.close((error) => (error ? reject(error) : resolve())))
  }
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
      const warnings: string[] = []
      const onWarning = (warning: Error) => warnings.push(warning.message)
      process.on('warning', onWarning)
      try {
        await run(create(root))
      } finally {
        process.off('warning', onWarning)
      }
      // Assets added after the compilation is sealed are deprecated.
      expect(warnings.filter((warning) => warning.includes('Compilation.assets'))).toEqual([])

      const dist = join(root, 'dist')
      expect(listFiles(dist)).toEqual([...DEFAULT_FILES, 'main.js'])
      expect(readFileSync(join(dist, 'main.js'), 'utf8')).toContain('https://cdn.example.com/favicon.svg')
    })
  })
}
