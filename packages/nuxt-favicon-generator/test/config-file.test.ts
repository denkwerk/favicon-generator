import { $fetch, setup } from '@nuxt/test-utils/e2e'
import { describe, expect, it } from 'vitest'
import { fixture, headTags } from './helpers'

describe('favicon.config.ts with inline options', async () => {
  await setup({ rootDir: fixture('config-file'), build: true, server: true })

  it('merges groups key by key, inline options winning', async () => {
    const manifest = await $fetch<Record<string, unknown>>('/manifest.json', { responseType: 'json' })
    expect(manifest).toMatchObject({ name: 'From File', short_name: 'Inline', display: 'browser', theme_color: '#222222' })
    expect(headTags(await $fetch<string>('/'))).toContain('<meta name="theme-color" content="#222222">')
  })
})
