import { readFileSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'
import { build, createServer } from 'vite'
import { describe, expect, it } from 'vitest'
import favicons from '../src/vite.js'
import { DEFAULT_FILES, copyFixture, headTags, listFiles } from './helpers.js'

describe('vite build', () => {
  it('writes the files to the output root and injects the tags', async () => {
    const root = copyFixture('app')
    await build({ root, logLevel: 'silent', plugins: [favicons({ input: './favicon.svg' })] })

    const dist = join(root, 'dist')
    expect(listFiles(dist).filter((file) => !file.startsWith('assets/'))).toEqual([...DEFAULT_FILES, 'index.html'])
    const tags = headTags(readFileSync(join(dist, 'index.html'), 'utf8'))
    expect(tags).toContain('<link rel="icon" type="image/svg+xml" href="/favicon.svg">')
    expect(tags).toContain('<link rel="apple-touch-icon" href="/apple-touch-icon.png">')
  })

  it('places the files under pathPrefix and prefixes the URLs with base', async () => {
    const root = copyFixture('app')
    await build({
      root,
      base: '/app/',
      logLevel: 'silent',
      plugins: [favicons({ input: './favicon.svg', pathPrefix: '/icons/', manifest: { name: 'Vite' } })],
    })

    const dist = join(root, 'dist')
    expect(listFiles(join(dist, 'icons'))).toEqual([
      'apple-touch-icon.png', 'favicon-192x192.png', 'favicon-512x512.png', 'favicon-96x96.png',
      'favicon.ico', 'favicon.svg', 'manifest.json',
    ])
    const html = readFileSync(join(dist, 'index.html'), 'utf8')
    expect(headTags(html)).toContain('<link rel="manifest" href="/app/icons/manifest.json">')
    const manifest = JSON.parse(readFileSync(join(dist, 'icons/manifest.json'), 'utf8'))
    expect(manifest.icons[0].src).toBe('/app/icons/favicon-192x192.png')
  })

  it('bundles the tags into the virtual module', async () => {
    const root = copyFixture('app')
    await build({ root, logLevel: 'silent', plugins: [favicons({ input: './favicon.svg' })] })

    const assets = listFiles(join(root, 'dist/assets')).filter((file) => file.endsWith('.js'))
    const code = assets.map((file) => readFileSync(join(root, 'dist/assets', file), 'utf8')).join('\n')
    expect(code).toContain('type="image/svg+xml" href="/favicon.svg"')
  })

  it('reads favicon.config.json and merges the inline options over it', async () => {
    const root = copyFixture('config-file')
    await build({
      root,
      logLevel: 'silent',
      plugins: [favicons({ manifest: { shortName: 'Inline' } })],
    })

    const manifest = JSON.parse(readFileSync(join(root, 'dist/manifest.json'), 'utf8'))
    expect(manifest).toMatchObject({ name: 'From the config file', short_name: 'Inline', theme_color: '#02969c' })
  })

  it('does not write the files into server builds', async () => {
    const root = copyFixture('app')
    await build({
      root,
      logLevel: 'silent',
      build: { ssr: './main.js', outDir: 'dist-ssr' },
      plugins: [favicons({ input: './favicon.svg' })],
    })

    expect(listFiles(join(root, 'dist-ssr'))).toEqual(['main.js'])
  })
})

describe('vite dev server', () => {
  it('serves the files and injects the tags', async () => {
    const root = copyFixture('app')
    const server = await createServer({
      root,
      logLevel: 'silent',
      server: { port: 0, ws: false },
      plugins: [favicons({ input: './favicon.svg' })],
    })
    await server.listen()
    try {
      const url = server.resolvedUrls!.local[0]!
      const svg = await fetch(new URL('favicon.svg', url))
      expect(svg.status).toBe(200)
      expect(svg.headers.get('content-type')).toBe('image/svg+xml')
      const ico = await fetch(new URL('favicon.ico', url))
      expect(ico.headers.get('content-type')).toBe('image/x-icon')

      const html = await (await fetch(url)).text()
      expect(headTags(html)).toContain('<link rel="icon" type="image/svg+xml" href="/favicon.svg">')

      const module = await server.ssrLoadModule('virtual:favicons')
      expect(module.link).toContainEqual({ rel: 'apple-touch-icon', href: '/apple-touch-icon.png' })
    } finally {
      await server.close()
    }
  })
  it('regenerates the files when the image changes', async () => {
    const root = copyFixture('app')
    const server = await createServer({
      root,
      logLevel: 'silent',
      server: { port: 0, ws: false },
      plugins: [favicons({ input: './favicon.svg' })],
    })
    await server.listen()
    try {
      const url = new URL('favicon.svg', server.resolvedUrls!.local[0]!)
      await fetch(url)
      const changed = readFileSync(join(root, 'favicon.svg'), 'utf8').replace('<svg', '<svg data-changed="true"')
      writeFileSync(join(root, 'favicon.svg'), changed)

      await expect.poll(async () => (await fetch(url)).text(), { timeout: 10_000 }).toContain('data-changed="true"')
    } finally {
      await server.close()
    }
  })
})
