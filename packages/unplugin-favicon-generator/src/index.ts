import { existsSync } from 'node:fs'
import { readFile } from 'node:fs/promises'
import { extname, join, relative } from 'node:path'
import { type UnpluginFactory, createUnplugin } from 'unplugin'
import { type Favicons, type Options, PLUGIN_NAME, joinUrl, normalizePrefix, prepareFavicons } from './core.js'

export type { Favicons, HeadTag, Options } from './core.js'

/**
 * Import it to render the tags yourself, e.g. in an SSR template. Types:
 * `@denkwerk/unplugin-favicon-generator/client`.
 */
export const VIRTUAL_MODULE_ID = 'virtual:favicons'
/** The same module for webpack and Rspack, which read `virtual:` as a URL scheme. Works everywhere. */
export const VIRTUAL_MODULE_ALIAS = '~favicons'
const RESOLVED_VIRTUAL_MODULE_ID = '\0favicons'

const CONTENT_TYPES: Record<string, string> = {
  '.ico': 'image/x-icon',
  '.png': 'image/png',
  '.svg': 'image/svg+xml',
  '.json': 'application/manifest+json',
  '.xml': 'application/xml',
}

/** Files that would be served in place of the generated ones in Vite's `publicDir`. */
const SHADOWED_FILES = ['favicon.ico', 'favicon.svg', 'apple-touch-icon.png', 'manifest.json']

export const unpluginFactory: UnpluginFactory<Options | undefined> = (options = {}) => {
  const prefix = normalizePrefix(options.pathPrefix)
  // Relative to the output directory: `''` for the root, `favicons/` for `/favicons/`.
  const outputDir = prefix.slice(1)

  let root = options.root ?? process.cwd()
  let base = options.base ?? '/'
  let serve = false
  let current: Promise<Favicons | null> | undefined

  const prepare = (warn: (message: string) => void) => {
    current = prepareFavicons(options, { root, urlPrefix: joinUrl(base, prefix), warn })
    return current
  }

  return {
    name: PLUGIN_NAME,

    async buildStart() {
      // Again on every (watch) build, so changes to the image are picked up;
      // unchanged output comes from the cache.
      const favicons = await prepare((message) => console.warn(`[${PLUGIN_NAME}] ${message}`))
      for (const file of favicons?.watchFiles ?? []) {
        this.addWatchFile(file)
      }
    },

    async buildEnd() {
      const favicons = await current
      if (serve || !favicons || isServerBuild(this)) {
        return
      }
      for (const file of favicons.files) {
        this.emitFile({ type: 'asset', fileName: `${outputDir}${file}`, source: await readFile(join(favicons.dir, file)) })
      }
    },

    resolveId(id) {
      return id === VIRTUAL_MODULE_ID || id === VIRTUAL_MODULE_ALIAS ? RESOLVED_VIRTUAL_MODULE_ID : undefined
    },

    async load(id) {
      if (id !== RESOLVED_VIRTUAL_MODULE_ID) {
        return undefined
      }
      const { link = [], meta = [], html = '' } = (await current) ?? {}
      return [
        `export const link = ${JSON.stringify(link)}`,
        `export const meta = ${JSON.stringify(meta)}`,
        `export const html = ${JSON.stringify(html)}`,
        'export default { link, meta, html }',
      ].join('\n')
    },

    vite: {
      configResolved(config) {
        root = options.root ?? config.root
        base = config.base
        serve = config.command === 'serve'
        const clashes = SHADOWED_FILES.filter((file) => config.publicDir && existsSync(join(config.publicDir, outputDir, file)))
        if (clashes.length > 0) {
          config.logger.warn(
            `[${PLUGIN_NAME}] ${clashes.map((file) => relative(root, join(config.publicDir, outputDir, file))).join(', ')} `
            + 'conflict with the generated favicons; remove them to use the generated ones.',
          )
        }
      },

      async transformIndexHtml() {
        const favicons = options.inject === false ? null : await current
        if (!favicons) {
          return []
        }
        return [
          ...favicons.link.map((attrs) => ({ tag: 'link', attrs, injectTo: 'head' as const })),
          ...favicons.meta.map((attrs) => ({ tag: 'meta', attrs, injectTo: 'head' as const })),
        ]
      },

      configureServer(server) {
        const warn = (message: string) => server.config.logger.warn(`[${PLUGIN_NAME}] ${message}`)
        const urlPrefix = joinUrl(server.config.base, prefix)

        // Before Vite's own middlewares, which would answer with index.html.
        server.middlewares.use(async (req, res, next) => {
          const path = req.url?.split('?')[0] ?? ''
          if (!path.startsWith(urlPrefix)) {
            return next()
          }
          try {
            const favicons = await (current ?? prepare(warn))
            const file = decodeURIComponent(path.slice(urlPrefix.length))
            if (!favicons?.files.includes(file)) {
              return next()
            }
            res.setHeader('Content-Type', CONTENT_TYPES[extname(file)] ?? 'application/octet-stream')
            res.setHeader('Cache-Control', 'no-cache')
            res.end(await readFile(join(favicons.dir, file)))
          } catch (error) {
            next(error)
          }
        })

        // Regenerates when the image or the config file changes, then reloads the page.
        server.watcher.on('change', async (file) => {
          const favicons = await current
          if (!favicons?.watchFiles.includes(file)) {
            return
          }
          try {
            const updated = await prepare(warn)
            server.watcher.add(updated?.watchFiles ?? [])
            for (const environment of Object.values(server.environments)) {
              const module = environment.moduleGraph.getModuleById(RESOLVED_VIRTUAL_MODULE_ID)
              if (module) {
                environment.moduleGraph.invalidateModule(module)
              }
            }
            server.config.logger.info(`[${PLUGIN_NAME}] regenerated favicons`, { timestamp: true })
            server.ws.send({ type: 'full-reload' })
          } catch (error) {
            server.config.logger.error(`[${PLUGIN_NAME}] ${(error as Error).message}`)
          }
        })

        void (current ?? prepare(warn)).then((favicons) => server.watcher.add(favicons?.watchFiles ?? []))
      },
    },
  }
}

/** Vite builds each environment separately; only the browser's output gets the files. */
function isServerBuild(context: object): boolean {
  return (context as { environment?: { config: { consumer: string } } }).environment?.config.consumer === 'server'
}

export const unplugin = /* #__PURE__ */ createUnplugin(unpluginFactory)

export default unplugin
