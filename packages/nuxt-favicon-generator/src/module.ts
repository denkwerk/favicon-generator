import { createHash } from 'node:crypto'
import { existsSync, readFileSync } from 'node:fs'
import { readdir, readFile, rename, rm } from 'node:fs/promises'
import { createRequire } from 'node:module'
import { dirname, join, relative, resolve } from 'node:path'
import { type FaviconConfig, generate, loadConfig } from '@denkwerk/favicon-generator'
import { defineNuxtModule, setGlobalHead, useLogger } from '@nuxt/kit'
import type { NuxtModule } from '@nuxt/schema'
import { joinURL, withLeadingSlash, withTrailingSlash } from 'ufo'

export interface ModuleOptions extends Omit<FaviconConfig, 'output' | 'overwrite' | 'snippets' | 'pathPrefix'> {
  /** Set to `false` to disable the module. @default true */
  enabled?: boolean
  /**
   * Path under `app.baseURL` that the files are served from. `/` serves
   * `/favicon.ico`, which browsers request even without a `<link>`. @default '/'
   */
  pathPrefix?: string
  /** Add the `<link>` and `<meta>` tags to every page. @default true */
  head?: boolean
  /**
   * Reuse the generated files while the options and the input file are
   * unchanged. Figma exports are cached too; set `false` to export on every
   * build. @default true
   */
  cache?: boolean
  /**
   * A `favicon.config.{js,ts,mjs,mts,cjs,cts,json}` is looked up like the CLI
   * does, starting in the Nuxt `rootDir`, and merged below the options set
   * here. Pass a path to use a specific file, or `false` to ignore config files.
   */
  configFile?: string | false
}

/** Options for the generator: everything except the module's own switches. */
type GeneratorOptions = Omit<ModuleOptions, 'enabled' | 'head' | 'cache' | 'configFile'>

const NAME = '@denkwerk/nuxt-favicon-generator'
const require = createRequire(import.meta.url)
const generatorVersion: string = require('@denkwerk/favicon-generator/package.json').version

const isFigmaUrl = (input: string) => /^(https?:\/\/)?(www\.)?figma\.com\//.test(input)

const module: NuxtModule<ModuleOptions> = defineNuxtModule<ModuleOptions>({
  meta: {
    name: NAME,
    configKey: 'favicon',
    compatibility: {
      nuxt: '>=3.0.0',
    },
  },
  // Generator options have no defaults here, so that a config file can set them.
  defaults: {
    enabled: true,
    head: true,
    cache: true,
  },
  async setup(options, nuxt) {
    const logger = useLogger('favicon-generator')
    const { enabled, head, cache, configFile, ...inlineOptions } = options
    if (!enabled) {
      return
    }

    const { rootDir } = nuxt.options
    // Type-check favicon.config.ts along with nuxt.config.ts.
    nuxt.options.typescript.nodeTsConfig ||= {}
    nuxt.options.typescript.nodeTsConfig.include ||= []
    nuxt.options.typescript.nodeTsConfig.include.push(join(relative(nuxt.options.buildDir, rootDir), 'favicon.config.*'))

    // `nuxt prepare` only generates types.
    if (nuxt.options._prepare) {
      return
    }

    const loaded = configFile === false
      ? { path: null, config: {} }
      : await loadConfig({ cwd: rootDir, configFile })
    if (loaded.path) {
      logger.info(`Using ${relative(rootDir, loaded.path)}`)
      nuxt.options.watch.push(loaded.path)
    }
    const generatorOptions = mergeOptions(loaded.config, inlineOptions)
    if (!generatorOptions.input) {
      logger.warn('No `input` is set in the `favicon` options or a favicon.config file, so no favicons are generated.')
      return
    }

    const isFigma = isFigmaUrl(generatorOptions.input)
    const input = isFigma ? generatorOptions.input : resolve(rootDir, generatorOptions.input)
    if (!isFigma && !existsSync(input)) {
      throw new Error(`[${NAME}] input file not found: ${input}`)
    }

    // Served by Nitro under app.baseURL; the generated URLs need both.
    const routePrefix = withTrailingSlash(withLeadingSlash(generatorOptions.pathPrefix ?? '/'))
    const config: FaviconConfig = {
      ...generatorOptions,
      input,
      figmaTokenFile: generatorOptions.figmaTokenFile && resolve(rootDir, generatorOptions.figmaTokenFile),
      pathPrefix: withTrailingSlash(joinURL(nuxt.options.app.baseURL, routePrefix)),
      overwrite: true,
      snippets: ['json'],
    }

    // The cache key covers everything that affects the output, but not the token.
    const hash = createHash('sha256')
      .update(JSON.stringify({ generatorVersion, config: { ...config, figmaToken: undefined } }))
      .update(isFigma ? '' : readFileSync(input))
      .digest('hex')
      .slice(0, 16)
    const cacheRoot = join(rootDir, 'node_modules/.cache/nuxt-favicon-generator')
    // <hash>/public is served; <hash>/head.json holds the tags.
    const publicDir = join(cacheRoot, hash, 'public')
    const headFile = join(cacheRoot, hash, 'head.json')
    config.output = publicDir

    warnAboutShadowedFiles(resolve(rootDir, nuxt.options.dir.public), routePrefix, logger)

    if (!isFigma) {
      nuxt.options.watch.push(input)
    }

    nuxt.hook('nitro:config', (nitroConfig) => {
      nitroConfig.publicAssets ||= []
      nitroConfig.publicAssets.push({ dir: publicDir, baseURL: routePrefix, maxAge: 60 * 60 * 24 })
      // The prerender crawler follows <link rel="manifest">; under a non-root
      // app.baseURL it would render the manifest route as a page and replace
      // the file with a directory.
      nitroConfig.prerender ||= {}
      nitroConfig.prerender.ignore ||= []
      nitroConfig.prerender.ignore.push(
        joinURL(routePrefix, 'manifest.json'),
        joinURL(nuxt.options.app.baseURL, routePrefix, 'manifest.json'),
      )
    })

    // Generating can take a moment (especially from Figma), so it runs after
    // module setup instead of delaying it.
    nuxt.hook('modules:done', async () => {
      if (!cache || !existsSync(headFile)) {
        const start = performance.now()
        await generate(config, { cwd: rootDir, silent: true })
        await rename(join(publicDir, 'favicon-head.json'), headFile)
        const source = isFigma ? 'Figma' : relative(rootDir, input)
        logger.success(`Generated favicons from ${source} in ${Math.round(performance.now() - start)}ms`)
        await removeStaleOutputs(cacheRoot, hash)
      }

      if (head) {
        const tags = JSON.parse(await readFile(headFile, 'utf8'))
        setGlobalHead({ link: tags.link, meta: tags.meta })
      }
    })
  },
})

export default module

/**
 * Inline module options take precedence over the config file, like CLI flags
 * do. The module decides where the files go, so the file's output settings
 * are dropped.
 */
function mergeOptions(fromFile: FaviconConfig, inline: GeneratorOptions): GeneratorOptions {
  const { output: _output, overwrite: _overwrite, snippets: _snippets, ...rest } = fromFile
  const defined = Object.fromEntries(Object.entries(inline).filter(([, value]) => value !== undefined))
  return { ...rest, ...defined }
}

/** Warns when files in `public/` would compete with the generated ones. */
function warnAboutShadowedFiles(publicDir: string, routePrefix: string, logger: ReturnType<typeof useLogger>) {
  const dir = join(publicDir, routePrefix)
  const clashes = ['favicon.ico', 'favicon.svg', 'apple-touch-icon.png', 'manifest.json']
    .filter((file) => existsSync(join(dir, file)))
  if (clashes.length > 0) {
    logger.warn(
      `${clashes.map((file) => relative(dirname(publicDir), join(dir, file))).join(', ')} `
      + 'conflict with the generated favicons; remove them to use the generated ones.',
    )
  }
}

async function removeStaleOutputs(cacheRoot: string, keep: string) {
  const entries = await readdir(cacheRoot).catch(() => [])
  await Promise.all(
    entries.filter((entry) => entry !== keep).map((entry) => rm(join(cacheRoot, entry), { recursive: true, force: true })),
  )
}
