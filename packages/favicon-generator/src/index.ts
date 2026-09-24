import { spawn } from 'node:child_process'
import { binaryEnv, resolveBinary } from './binary.js'

export type Snippet = 'html' | 'nuxt' | 'json'
export type Display = 'fullscreen' | 'standalone' | 'minimal-ui' | 'browser'
/** `#rgb` or `#rrggbb`; validated when the favicons are generated. */
export type HexColor = string

/** Options of the Apple touch icon. */
export interface AppleTouchIconOptions {
  /** Color behind transparent pixels; iOS shows them black. @default '#ffffff' */
  background?: HexColor
}

/** Options of the web app manifest, which contains exactly the fields you set. */
export interface ManifestOptions {
  /** `name`, shown when installing the app. */
  name?: string
  /** `short_name`, shown on the home screen. */
  shortName?: string
  /** `description`. */
  description?: string
  /** `background_color` of the splash screen; also behind maskable icons. */
  backgroundColor?: HexColor
  /** `start_url`. */
  startUrl?: string
  /** `scope`. */
  scope?: string
  /** `display` mode. */
  display?: Display
  /** Add maskable icons (the image at 60% on `backgroundColor`) for Android. @default false */
  maskable?: boolean
  /** `crossorigin` attribute for the manifest `<link>`, e.g. `use-credentials`. */
  crossorigin?: string
}

/** Options of the Windows tiles. */
export interface WindowsOptions {
  /** Tile color (`msapplication-TileColor`). */
  tileColor?: HexColor
}

/**
 * Options for favicon-generator. Every option has a CLI flag, and flags take
 * precedence. Relative paths are resolved against the config file's directory.
 *
 * With `input` and `output`, you get `favicon.ico`, `favicon.svg`,
 * `favicon-96x96.png`, `apple-touch-icon.png` and `favicon.html`. Additionally,
 * the groups below add a manifest, Windows tiles and more: set one to `true` for
 * its defaults or to its options; `false` leaves it out.
 */
export interface FaviconConfig {
  /** Path or URL of the JSON Schema, for editor support in `favicon.config.json`. */
  $schema?: string
  /**
   * Source image (SVG, PNG, JPEG or WebP; should be square), or a Figma link
   * with a `node-id`, which is exported as SVG through the Figma REST API.
   */
  input?: string
  /** Directory to write the generated files into. @default 'favicons' */
  output?: string
  /** Overwrite existing files in the output directory. @default false */
  overwrite?: boolean
  /** URL prefix the files will be served from, e.g. `/favicons/`. @default '/' */
  pathPrefix?: string
  /** Head snippets to write next to the icons; `[]` writes none. @default ['html'] */
  snippets?: Snippet[]
  /** Adds `<meta name="theme-color">` (and `theme_color` to the manifest). */
  themeColor?: HexColor
  /** `apple-touch-icon.png`, 180×180 and opaque. Included by default; `false` leaves it out. */
  appleTouchIcon?: boolean | AppleTouchIconOptions
  /** Adds a web app manifest (`manifest.json`) with 192 and 512 px icons: `true` or its options. */
  manifest?: boolean | ManifestOptions
  /** Adds `browserconfig.xml` and tile images for pinned sites on Windows: `true` or its options. */
  windows?: boolean | WindowsOptions
  /**
   * Also generate the sizes old browsers and devices look for: 19 PNG sizes,
   * sized Apple touch icons and a 7-frame `favicon.ico`. @default false
   */
  legacy?: boolean
  /**
   * Reuse the generated files while the input and the options are unchanged,
   * and a Figma export while the Figma file's version is unchanged. @default true
   */
  cache?: boolean
  /**
   * Directory for the cache. @default 'node_modules/.cache/favicon-generator'
   * in the project root, if it has a `node_modules` directory
   */
  cacheDir?: string
  /**
   * Figma personal access token (scope `file_content:read`). Prefer the
   * `FIGMA_TOKEN` env var or `figmaTokenFile` over committing a token.
   */
  figmaToken?: string
  /** File containing the Figma personal access token. */
  figmaTokenFile?: string
}

export type UserConfig = FaviconConfig | (() => FaviconConfig | Promise<FaviconConfig>)

/**
 * Identity helper that gives `favicon.config.ts` type checking and editor
 * completion. Not generic on purpose: that would skip excess property checks,
 * so typos like `themeColr` would go unnoticed. TypeScript does not check a
 * function's returned object for unknown keys; annotate its return type
 * (`(): FaviconConfig => ({ ... })`) to get the same checks.
 */
export function defineConfig(config: FaviconConfig): FaviconConfig
export function defineConfig(config: () => FaviconConfig | Promise<FaviconConfig>): () => FaviconConfig | Promise<FaviconConfig>
export function defineConfig(config: UserConfig): UserConfig {
  return config
}

/** Options that are groups: `true`, `false` or an options object. */
const GROUPS = new Set(['appleTouchIcon', 'manifest', 'windows'])

const isObject = (value: unknown): value is Record<string, unknown> =>
  typeof value === 'object' && value !== null && !Array.isArray(value)

/**
 * Merges `overrides` over `base` the way CLI flags override a config file:
 * `undefined` values are skipped, and the options of a group are merged key
 * by key, so `{ manifest: { name } }` keeps the file's other manifest options.
 */
export function mergeConfig<T extends FaviconConfig>(base: T, overrides: Partial<T>): T {
  const result = { ...base } as Record<string, unknown>
  for (const [key, value] of Object.entries(overrides)) {
    if (value === undefined) {
      continue
    }
    const current = result[key]
    result[key] = GROUPS.has(key) && isObject(current) && isObject(value) ? { ...current, ...value } : value
  }
  return result as T
}

export interface GenerateOptions {
  /** Directory that relative paths in `config` are resolved against. @default process.cwd() */
  cwd?: string
  /** Suppress progress output; warnings and errors are still printed. @default false */
  silent?: boolean
}

/**
 * Generates favicons programmatically. Config files are not read; `config` is
 * the complete configuration.
 */
export async function generate(config: FaviconConfig, options: GenerateOptions = {}): Promise<void> {
  const child = spawn(resolveBinary(), ['--config', '-'], {
    cwd: options.cwd,
    env: binaryEnv(),
    stdio: ['pipe', options.silent ? 'ignore' : 'inherit', 'inherit'],
  })
  child.stdin.end(JSON.stringify(config))

  await new Promise<void>((resolve, reject) => {
    child.once('error', reject)
    child.once('close', (code, signal) => {
      if (code === 0) {
        resolve()
      } else {
        reject(new Error(`favicon-generator failed (${signal ?? `exit code ${code}`})`))
      }
    })
  })
}

export interface LoadConfigOptions {
  /** Directory to start the lookup from. @default process.cwd() */
  cwd?: string
  /** Use this config file instead of looking one up (relative to `cwd`). */
  configFile?: string
}

export interface LoadedConfig {
  /** Absolute path of the config file, or `null` if none was found. */
  path: string | null
  /** The config's default export, with paths made absolute. */
  config: FaviconConfig
}

/**
 * Finds and evaluates a `favicon.config.{js,ts,mjs,mts,cjs,cts,json}` with
 * exactly the rules of the CLI: in `cwd`, then its parents up to the nearest
 * directory containing a `package.json` or `.git`.
 */
export async function loadConfig(options: LoadConfigOptions = {}): Promise<LoadedConfig> {
  const args = ['--print-config', ...(options.configFile ? ['--config', options.configFile] : [])]
  const child = spawn(resolveBinary(), args, {
    cwd: options.cwd,
    env: binaryEnv(),
    stdio: ['ignore', 'pipe', 'inherit'],
  })
  let stdout = ''
  child.stdout.setEncoding('utf8').on('data', (chunk: string) => {
    stdout += chunk
  })

  await new Promise<void>((resolve, reject) => {
    child.once('error', reject)
    child.once('close', (code, signal) => {
      if (code === 0) {
        resolve()
      } else {
        reject(new Error(`favicon-generator could not load the config (${signal ?? `exit code ${code}`})`))
      }
    })
  })
  return JSON.parse(stdout) as LoadedConfig
}
