import { $fetch, fetch, setup } from '@nuxt/test-utils/e2e'
import { describe, expect, it } from 'vitest'
import { fixture, headTags, isFile } from './helpers'

describe('mirrorPrefixes and the manifest icons', async () => {
  await setup({ rootDir: fixture('mirror'), build: true, server: true })

  it('serves the files under pathPrefix and every mirror', async () => {
    for (const file of ['favicon.ico', 'favicon.svg', 'favicon-72x72.png', 'apple-touch-icon.png', 'manifest.json']) {
      expect(isFile(await fetch(`/public/${file}`)), `/public/${file}`).toBe(true)
      expect(isFile(await fetch(`/${file}`)), `/${file}`).toBe(true)
    }
  })

  it('links the files under pathPrefix', async () => {
    const tags = headTags(await $fetch<string>('/'))
    expect(tags).toContain('<link rel="icon" type="image/svg+xml" href="/public/favicon.svg">')
    expect(tags).toContain('<link rel="manifest" href="/public/manifest.json">')
    expect(tags.join('\n')).not.toMatch(/href="\/favicon/)
  })

  it('lists the configured icon sizes with their purpose', async () => {
    const manifest = await $fetch<{ icons: { src: string, sizes: string, purpose?: string }[] }>('/public/manifest.json', { responseType: 'json' })
    expect(manifest.icons).toEqual([72, 192, 512].map((size) => ({
      src: `/public/favicon-${size}x${size}.png`,
      type: 'image/png',
      sizes: `${size}x${size}`,
      purpose: 'any maskable',
    })))
  })
})
