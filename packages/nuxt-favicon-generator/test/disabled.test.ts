import { $fetch, fetch, setup } from '@nuxt/test-utils/e2e'
import { describe, expect, it } from 'vitest'
import { fixture, headTags, isFile } from './helpers'

describe('enabled: false', async () => {
  await setup({ rootDir: fixture('disabled'), build: true, server: true })

  it('generates and links nothing', async () => {
    expect(isFile(await fetch('/favicon-96x96.png'))).toBe(false)
    expect(headTags(await $fetch<string>('/')).join('\n')).not.toContain('favicon')
  })
})
