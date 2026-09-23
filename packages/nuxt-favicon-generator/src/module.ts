import { createHash } from 'node:crypto'
import { existsSync, readFileSync } from 'node:fs'
import { readdir, readFile, rename, rm } from 'node:fs/promises'
import { createRequire } from 'node:module'
import { dirname, join, relative, resolve } from 'node:path'
import { type FaviconConfig, generate } from '@denkwerk/favicon-generator'
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
}

const NAME = '@denkwerk/nuxt-favicon-generator'
const require = createRequire(import.meta.url)
const generatorVersion: string = require('@denkwerk/favicon-generator/package.json').version

const isFigmaUrl = (input: string) => /^(https?:\/\/)?(www\.)?figma\.com\//.test(input)

const module: NuxtModule<ModuleOptions> = defineNuxtModule<ModuleOptions>({
  meta: {
    name: NAME,
    configKey: 'faviconGenerator',
    compatibility: {
      nuxt: '>=3.0.0',
    },
  },
  defaults: {
    enabled: true,
    pathPrefix: '/',
    head: true,
    cache: true,
  },
  setup(options, nuxt) {
    const logger = useLogger('favicon-generator')
    const { enabled, head, cache, pathPrefix = '/', ...generatorOptions } = options
    if (!enabled) {
      return
    }
    if (!generatorOptions.input) {
      logger.warn('`faviconGenerator.input` is not set, so no favicons are generated.')
      return
    }

    const { rootDir } = nuxt.options
    const isFigma = isFigmaUrl(generatorOptions.input)
    const input = isFigma ? generatorOptions.input : resolve(rootDir, generatorOptions.input)
    if (!isFigma && !existsSync(input)) {
      throw new Error(`[${NAME}] input file not found: ${input}`)
    }

    // Served by Nitro under app.baseURL; the generated URLs need both.
    const routePrefix = withTrailingSlash(withLeadingSlash(pathPrefix))
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
    // module setup instead of delaying it. `nuxt prepare` only needs types.
    nuxt.hook('modules:done', async () => {
      if (nuxt.options._prepare) {
        return
      }

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
