import { $fetch, fetch, setup } from '@nuxt/test-utils/e2e'
import { describe, expect, it } from 'vitest'
import { fixture, headTags, isFile } from './helpers'

describe('defaults', async () => {
  await setup({ rootDir: fixture('basic'), build: true, server: true })

  it('serves the minimal set from the site root', async () => {
    for (const file of ['/favicon.ico', '/favicon.svg', '/favicon-96x96.png', '/apple-touch-icon.png']) {
      expect(isFile(await fetch(file)), file).toBe(true)
    }
    expect(isFile(await fetch('/manifest.json'))).toBe(false)
  })

  it('adds only the icon tags', async () => {
    const tags = headTags(await $fetch<string>('/'))
    expect(tags).toContain('<link rel="icon" href="/favicon.ico" sizes="32x32">')
    expect(tags).toContain('<link rel="icon" type="image/svg+xml" href="/favicon.svg">')
    expect(tags).toContain('<link rel="icon" type="image/png" sizes="96x96" href="/favicon-96x96.png">')
    expect(tags).toContain('<link rel="apple-touch-icon" href="/apple-touch-icon.png">')
    expect(tags.join('\n')).not.toMatch(/manifest|theme-color|msapplication/)
  })
})
