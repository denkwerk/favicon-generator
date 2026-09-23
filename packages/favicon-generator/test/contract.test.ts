// The TypeScript types are written by hand (for their docs); the JSON Schema is
// generated from the Rust config types. This keeps the two in sync: each list
// must name every key of its TS type (checked by `tsc -p test`), and the same
// keys must be in the schema (checked here).
import { describe, expect, it } from 'vitest'
import type { AppleTouchIconOptions, FaviconConfig, ManifestOptions, WindowsOptions } from '../src/index.js'
import schema from '../schema.json' with { type: 'json' }

type Keys<T> = Record<keyof Required<T>, true>

const config = {
  $schema: true,
  input: true,
  output: true,
  overwrite: true,
  pathPrefix: true,
  snippets: true,
  themeColor: true,
  appleTouchIcon: true,
  manifest: true,
  windows: true,
  legacy: true,
  figmaToken: true,
  figmaTokenFile: true,
} satisfies Keys<FaviconConfig>

const appleTouchIcon = { background: true } satisfies Keys<AppleTouchIconOptions>

const manifest = {
  name: true,
  shortName: true,
  description: true,
  backgroundColor: true,
  startUrl: true,
  scope: true,
  display: true,
  maskable: true,
  crossorigin: true,
} satisfies Keys<ManifestOptions>

const windows = { tileColor: true } satisfies Keys<WindowsOptions>

const definitions = schema.definitions as Record<string, { properties?: Record<string, unknown>, enum?: string[], oneOf?: { const?: string, enum?: string[] }[] }>
const keysOf = (properties: Record<string, unknown> | undefined) => Object.keys(properties ?? {}).sort()

describe('the TypeScript types match the JSON Schema', () => {
  it.each([
    ['FaviconConfig', config, schema.properties],
    ['AppleTouchIconOptions', appleTouchIcon, definitions.AppleTouchIconConfig?.properties],
    ['ManifestOptions', manifest, definitions.ManifestConfig?.properties],
    ['WindowsOptions', windows, definitions.WindowsConfig?.properties],
  ])('%s', (_name, keys, properties) => {
    expect(Object.keys(keys).sort()).toEqual(keysOf(properties))
  })

  it('Display', () => {
    const display = ['fullscreen', 'standalone', 'minimal-ui', 'browser'] satisfies ManifestOptions['display'][]
    const values = definitions.Display?.oneOf?.flatMap((variant) => variant.enum ?? [variant.const]) ?? definitions.Display?.enum
    expect([...display].sort()).toEqual([...(values ?? [])].sort())
  })
})
