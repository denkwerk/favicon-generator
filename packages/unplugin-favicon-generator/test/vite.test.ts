import { readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs'
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

  it('writes the files to every mirror and links them under pathPrefix', async () => {
    const root = copyFixture('app')
    await build({
      root,
      logLevel: 'silent',
      plugins: [favicons({ input: './favicon.svg', pathPrefix: '/public/', mirrorPrefixes: ['/'] })],
    })

    const dist = join(root, 'dist')
    const files = listFiles(dist)
    for (const file of DEFAULT_FILES) {
      expect(files).toContain(`public/${file}`)
      expect(files).toContain(file)
    }
    const tags = headTags(readFileSync(join(dist, 'index.html'), 'utf8'))
    expect(tags).toContain('<link rel="icon" type="image/svg+xml" href="/public/favicon.svg">')
    expect(tags.join('\n')).not.toMatch(/href="\/favicon/)
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

describe('cache', () => {
  const cacheDir = (root: string) => join(root, 'node_modules/.cache/favicon-generator')
  const buildWith = (root: string, options: Parameters<typeof favicons>[0]) =>
    build({ root, logLevel: 'silent', plugins: [favicons({ input: './favicon.svg', ...options })] })

  it('keeps the output in node_modules/.cache/favicon-generator and reuses it', async () => {
    const root = copyFixture('app')
    await buildWith(root, {})
    const [entry] = readdirSync(join(cacheDir(root), 'unplugin'))
    const icon = join(cacheDir(root), 'unplugin', entry!, 'files/favicon.ico')
    const generated = statSync(icon).mtimeMs
    // The generator's own cache.
    expect(readdirSync(join(cacheDir(root), 'outputs'))).toHaveLength(1)

    await buildWith(root, {})
    expect(statSync(icon).mtimeMs).toBe(generated)

    // A change to the image is a new entry; the old one is removed.
    writeFileSync(join(root, 'favicon.svg'), readFileSync(join(root, 'favicon.svg'), 'utf8').replace('<svg', '<svg data-changed="1"'))
    await buildWith(root, {})
    expect(readdirSync(join(cacheDir(root), 'unplugin'))).not.toContain(entry)
  })

  it('generates again with cache: false, and cacheDir moves the cache', async () => {
    const root = copyFixture('app')
    await buildWith(root, { cacheDir: '.favicons' })
    const [entry] = readdirSync(join(root, '.favicons/unplugin'))
    const icon = join(root, '.favicons/unplugin', entry!, 'files/favicon.ico')
    const generated = statSync(icon).mtimeMs

    await buildWith(root, { cacheDir: '.favicons', cache: false })
    expect(statSync(icon).mtimeMs).toBeGreaterThan(generated)
    expect(listFiles(join(root, 'dist')).filter((file) => !file.startsWith('assets/'))).toEqual([...DEFAULT_FILES, 'index.html'])
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
  it('serves the files under every mirror', async () => {
    const root = copyFixture('app')
    const server = await createServer({
      root,
      logLevel: 'silent',
      server: { port: 0, ws: false },
      plugins: [favicons({ input: './favicon.svg', pathPrefix: '/public/', mirrorPrefixes: ['/'] })],
    })
    await server.listen()
    try {
      const url = server.resolvedUrls!.local[0]!
      for (const path of ['public/favicon.svg', 'favicon.svg', 'public/favicon.ico', 'favicon.ico']) {
        const response = await fetch(new URL(path, url))
        expect(response.status, path).toBe(200)
        expect(response.headers.get('content-type'), path).toMatch(/^image\//)
      }
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
