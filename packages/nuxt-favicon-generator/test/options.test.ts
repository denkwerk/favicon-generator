import { describe, expect, it } from 'vitest'
import { mergeOptions } from '../src/options'

describe('mergeOptions', () => {
  it('lets inline options win and merges groups key by key', () => {
    const merged = mergeOptions(
      { input: 'file.svg', themeColor: '#111111', manifest: { name: 'File', display: 'browser' }, windows: true },
      { themeColor: '#222222', manifest: { shortName: 'Inline' }, windows: undefined },
    )
    expect(merged).toEqual({
      input: 'file.svg',
      themeColor: '#222222',
      manifest: { name: 'File', display: 'browser', shortName: 'Inline' },
      windows: true,
    })
  })

  it('replaces a group with a boolean', () => {
    expect(mergeOptions({ manifest: { name: 'File' } }, { manifest: false }).manifest).toBe(false)
    expect(mergeOptions({ manifest: false }, { manifest: { name: 'Inline' } }).manifest).toEqual({ name: 'Inline' })
  })

  it('drops the output settings of the config file', () => {
    expect(mergeOptions({ output: 'x', overwrite: true, snippets: ['html'], $schema: 'y', legacy: true }, {})).toEqual({ legacy: true })
  })
})
