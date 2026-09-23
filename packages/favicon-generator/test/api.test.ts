import { readFileSync } from 'node:fs'
import { join } from 'node:path'
import { describe, expect, it } from 'vitest'
import { generate, loadConfig } from '../src/index.js'
import { createProject, list } from './helpers.js'

describe('generate', () => {
  it('writes the minimal set by default', async () => {
    const cwd = createProject()
    await generate({ input: 'logo.svg', output: 'out' }, { cwd, silent: true })
    expect(list(join(cwd, 'out'))).toEqual([
      'apple-touch-icon.png',
      'favicon-96x96.png',
      'favicon.html',
      'favicon.ico',
      'favicon.svg',
    ])
  })

  it('adds the configured groups', async () => {
    const cwd = createProject()
    await generate(
      {
        input: 'logo.svg',
        output: 'out',
        pathPrefix: '/icons/',
        themeColor: '#02969c',
        manifest: { name: 'My App', display: 'standalone' },
        windows: true,
        appleTouchIcon: false,
      },
      { cwd, silent: true },
    )
    const files = list(join(cwd, 'out'))
    expect(files).toContain('manifest.json')
    expect(files).toContain('browserconfig.xml')
    expect(files).not.toContain('apple-touch-icon.png')

    const manifest = JSON.parse(readFileSync(join(cwd, 'out/manifest.json'), 'utf8'))
    expect(manifest).toMatchObject({ name: 'My App', display: 'standalone', theme_color: '#02969c' })
    expect(manifest.icons[0].src).toBe('/icons/favicon-192x192.png')

    const html = readFileSync(join(cwd, 'out/favicon.html'), 'utf8')
    expect(html).toContain('<meta name="theme-color" content="#02969c">')
    expect(html).toContain('<meta name="msapplication-config" content="/icons/browserconfig.xml">')
  })

  it('does not read config files', async () => {
    const cwd = createProject({ 'favicon.config.json': '{ "manifest": { "name": "From file" } }' })
    await generate({ input: 'logo.svg', output: 'out' }, { cwd, silent: true })
    expect(list(join(cwd, 'out'))).not.toContain('manifest.json')
  })

  it('rejects the options renamed in 0.2.0', async () => {
    const cwd = createProject()
    // @ts-expect-error appName moved to manifest.name
    const run = generate({ input: 'logo.svg', output: 'out', appName: 'Old' }, { cwd, silent: true })
    await expect(run).rejects.toThrow(/favicon-generator failed/)
  })
})

describe('loadConfig', () => {
  it('finds and evaluates a TypeScript config', async () => {
    const cwd = createProject({
      'favicon.config.ts': `
        import type { FaviconConfig } from '${join(import.meta.dirname, '../src/index.ts')}'
        export default (): FaviconConfig => ({ input: './logo.svg', manifest: { name: 'From TS' } })
      `,
    })
    const { path, config } = await loadConfig({ cwd })
    expect(path).toBe(join(cwd, 'favicon.config.ts'))
    expect(config.input).toBe(join(cwd, 'logo.svg'))
    expect(config.manifest).toEqual({ name: 'From TS' })
  })

  it('returns null when there is no config file', async () => {
    const { path, config } = await loadConfig({ cwd: createProject() })
    expect(path).toBeNull()
    expect(config).toEqual({})
  })

  it('uses an explicit config file', async () => {
    const cwd = createProject({ 'custom.json': '{ "legacy": true }' })
    const { path, config } = await loadConfig({ cwd, configFile: 'custom.json' })
    expect(path).toBe(join(cwd, 'custom.json'))
    expect(config.legacy).toBe(true)
  })
})
