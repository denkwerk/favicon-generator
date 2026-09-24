import { createHash } from 'node:crypto'
import { existsSync, readFileSync } from 'node:fs'
import { readdir, readFile, rename, rm } from 'node:fs/promises'
import { createRequire } from 'node:module'
import { join, resolve } from 'node:path'
import { type FaviconConfig, generate, loadConfig, mergeConfig } from '@denkwerk/favicon-generator'

export interface Options extends Omit<FaviconConfig, '$schema' | 'output' | 'overwrite' | 'snippets'> {
  /**
   * Directory in the build output, and URL path below `base`, that the files
   * are written to and served from. `/` serves `/favicon.ico`, which browsers
   * request even without a `<link>`. @default '/'
   */
  pathPrefix?: string
  /**
   * URL the build output is served from. Vite uses its own `base` instead.
   * @default '/'
   */
  base?: string
  /**
   * Directory that the config file lookup and relative paths start from, and
   * that holds the cache in `node_modules/.cache/favicon-generator`.
   * @default Vite's `root`, else `process.cwd()`
   */
  root?: string
  /** Vite: add the `<link>` and `<meta>` tags to every HTML page. @default true */
  inject?: boolean
  /**
   * Reuse the generated files while the options and the input file are
   * unchanged. A Figma export is reused while the Figma file's version is
   * unchanged, which one small API request checks on every build; set `false`
   * to export on every build. @default true
   */
  cache?: boolean
  /**
   * A `favicon.config.{js,ts,mjs,mts,cjs,cts,json}` is looked up like the CLI
   * does, starting in the project root, and merged below the options set
   * here. Pass a path to use a specific file, or `false` to ignore config files.
   */
  configFile?: string | false
}

/** Options for the generator: everything except the plugin's own switches. */
type GeneratorOptions = Omit<Options, 'root' | 'base' | 'inject' | 'configFile'>

export interface HeadTag {
  [attribute: string]: string
}

export interface Favicons {
  /** Directory that holds the generated files. */
  dir: string
  /** The generated files, relative to `dir`; head snippets are not included. */
  files: string[]
  link: HeadTag[]
  meta: HeadTag[]
  /** The tags as HTML, one per line. */
  html: string
  /** Files whose changes affect the output: the config file and the input image. */
  watchFiles: string[]
}

export const PLUGIN_NAME = '@denkwerk/unplugin-favicon-generator'
const HEAD_FILE = 'favicon-head.json'
const HTML_FILE = 'favicon.html'
/** Shared with the generator's own cache. */
const CACHE_DIR = 'node_modules/.cache/favicon-generator'

const require = createRequire(import.meta.url)
const generatorVersion: string = require('@denkwerk/favicon-generator/package.json').version

/** Generations in progress, by their `favicon-head.json`. */
const inflight = new Map<string, Promise<void>>()

const isFigmaUrl = (input: string) => /^(https?:\/\/)?(www\.)?figma\.com\//.test(input)

/** `/`, `/favicons/`: leading and trailing slash. */
export const normalizePrefix = (prefix = '/') => `/${prefix.replace(/^\/+|\/+$/g, '')}/`.replace(/^\/\/$/, '/')

/** Joins a base URL (absolute, root-relative or `./`) and a normalized prefix. */
export const joinUrl = (base: string, prefix: string) => `${base.replace(/\/+$/, '')}${prefix}`

/**
 * Loads the config, generates the favicons into a cache directory keyed by
 * everything that affects the output, and reads the result back. Returns
 * `null` (after a warning) when no `input` is configured.
 */
export async function prepareFavicons(
  options: Options,
  { root, urlPrefix, warn }: { root: string, urlPrefix: string, warn: (message: string) => void },
): Promise<Favicons | null> {
  const { root: _root, base: _base, inject: _inject, configFile, ...inline } = options
  const loaded = configFile === false
    ? { path: null, config: {} }
    : await loadConfig({ cwd: root, configFile })
  const { output: _output, overwrite: _overwrite, snippets: _snippets, $schema: _schema, ...fromFile } = loaded.config
  const generatorOptions = mergeConfig<GeneratorOptions>(fromFile, inline)
  const cache = generatorOptions.cache ?? true
  const watchFiles = loaded.path ? [loaded.path] : []

  if (!generatorOptions.input) {
    warn('No `input` is set in the plugin options or a favicon.config file, so no favicons are generated.')
    return null
  }
  const isFigma = isFigmaUrl(generatorOptions.input)
  const input = isFigma ? generatorOptions.input : resolve(root, generatorOptions.input)
  if (!isFigma) {
    if (!existsSync(input)) {
      throw new Error(`[${PLUGIN_NAME}] input file not found: ${input}`)
    }
    watchFiles.push(input)
  }

  const cacheRoot = resolve(root, generatorOptions.cacheDir ?? CACHE_DIR)
  const config: FaviconConfig = {
    ...generatorOptions,
    input,
    cache,
    cacheDir: cacheRoot,
    figmaTokenFile: generatorOptions.figmaTokenFile && resolve(root, generatorOptions.figmaTokenFile),
    pathPrefix: urlPrefix,
    overwrite: true,
    snippets: ['json', 'html'],
  }

  // The key covers everything that affects the output, but not the token or
  // where the cache is.
  // A Figma export can change without the options changing, so the generator
  // checks it on every build (and reuses what it can from its own cache).
  const hash = createHash('sha256')
    .update(JSON.stringify({ generatorVersion, config: { ...config, figmaToken: undefined, cache: undefined, cacheDir: undefined } }))
    .update(isFigma ? '' : readFileSync(input))
    .digest('hex')
    .slice(0, 16)
  const outputsRoot = join(cacheRoot, 'unplugin')
  // <hash>/files holds the icons; the snippets are moved next to it, so that
  // `files` is exactly what gets served.
  const dir = join(outputsRoot, hash, 'files')
  const headFile = join(outputsRoot, hash, HEAD_FILE)
  const htmlFile = join(outputsRoot, hash, HTML_FILE)

  if (!cache || isFigma || !existsSync(headFile)) {
    // Builds and the dev server's watcher can ask for the same output at once.
    let pending = inflight.get(headFile)
    if (!pending) {
      pending = (async () => {
        await generate({ ...config, output: dir }, { cwd: root, silent: true })
        await rename(join(dir, HTML_FILE), htmlFile)
        // Last, so an interrupted run is not taken for a complete one.
        await rename(join(dir, HEAD_FILE), headFile)
        await removeStaleOutputs(outputsRoot, hash)
      })().finally(() => inflight.delete(headFile))
      inflight.set(headFile, pending)
    }
    await pending
  }

  const head = JSON.parse(await readFile(headFile, 'utf8')) as { link: HeadTag[], meta: HeadTag[] }
  return {
    dir,
    files: (await readdir(dir)).sort(),
    link: head.link,
    meta: head.meta,
    html: (await readFile(htmlFile, 'utf8')).trim(),
    watchFiles,
  }
}

async function removeStaleOutputs(cacheRoot: string, keep: string) {
  const entries = await readdir(cacheRoot).catch(() => [])
  await Promise.all(
    entries.filter((entry) => entry !== keep).map((entry) => rm(join(cacheRoot, entry), { recursive: true, force: true })),
  )
}
