import { $fetch, fetch, setup } from '@nuxt/test-utils/e2e'
import { describe, expect, it } from 'vitest'
import { fixture, headTags, isFile } from './helpers'

describe('every group, under app.baseURL', async () => {
  await setup({ rootDir: fixture('full'), build: true, server: true })

  it('serves the files under baseURL and pathPrefix', async () => {
    for (const file of ['favicon.ico', 'manifest.json', 'browserconfig.xml', 'favicon-maskable-512x512.png']) {
      expect(isFile(await fetch(`/app/icons/${file}`)), file).toBe(true)
    }
  })

  it('writes the manifest with the configured fields', async () => {
    const manifest = await $fetch<Record<string, unknown>>('/app/icons/manifest.json', { responseType: 'json' })
    expect(manifest).toMatchObject({
      name: 'Full Fixture',
      short_name: 'Full',
      display: 'standalone',
      theme_color: '#02969c',
    })
    expect((manifest.icons as { src: string }[]).map((icon) => icon.src)).toContain('/app/icons/favicon-192x192.png')
  })

  it('links everything from the page', async () => {
    const tags = headTags(await $fetch<string>('/app/'))
    expect(tags).toContain('<link rel="manifest" href="/app/icons/manifest.json">')
    expect(tags).toContain('<meta name="theme-color" content="#02969c">')
    expect(tags).toContain('<meta name="msapplication-TileColor" content="#02969c">')
  })
})
