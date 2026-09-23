import { spawn } from 'node:child_process'
import { binaryEnv, resolveBinary } from './binary.js'

export type Snippet = 'html' | 'nuxt'
export type Display = 'fullscreen' | 'standalone' | 'minimal-ui' | 'browser'
/** `#rgb` or `#rrggbb`. */
export type HexColor = `#${string}`

/**
 * Options for favicon-generator. Every option mirrors a CLI flag; flags passed
 * on the command line take precedence. Relative paths are resolved against the
 * directory of the config file.
 */
export interface FaviconConfig {
  /**
   * Source image (SVG, PNG, JPEG or WebP; should be square), or a Figma link
   * with a `node-id`, which is exported as SVG through the Figma REST API.
   */
  input?: string
  /** Directory to write the generated files into. @default 'favicons' */
  output?: string
  /** Overwrite existing files in the output directory. @default false */
  overwrite?: boolean
  /** URL prefix the files will be served from, e.g. `/public/`. @default '/' */
  pathPrefix?: string
  /** Application name used in manifest.json. @default 'App' */
  appName?: string
  /** Short application name. @default appName */
  appShortName?: string
  /** Application description. @default appName */
  appDescription?: string
  /** Browser UI color (`theme-color`, manifest `theme_color`). @default '#ffffff' */
  themeColor?: HexColor
  /**
   * Splash screen color (manifest `background_color`); also fills transparent
   * pixels in the opaque apple-touch-icon*.png files. @default '#ffffff'
   */
  backgroundColor?: HexColor
  /** Windows tile color. @default backgroundColor */
  tileColor?: HexColor
  /** Manifest `start_url`. @default '/?source=pwa' */
  startUrl?: string
  /** Manifest `scope`. @default '/' */
  scope?: string
  /** Manifest `display` mode. @default 'standalone' */
  display?: Display
  /** Manifest icon `purpose`. @default 'any maskable' */
  iconPurpose?: string
  /** `crossorigin` attribute for the manifest `<link>`, e.g. `use-credentials`. */
  manifestCrossorigin?: string
  /**
   * Figma personal access token (scope `file_content:read`). Prefer the
   * `FIGMA_TOKEN` env var or `figmaTokenFile` over committing a token.
   */
  figmaToken?: string
  /** File containing the Figma personal access token. */
  figmaTokenFile?: string
  /** Head snippets to write next to the icons; `[]` writes none. @default ['html', 'nuxt'] */
  snippets?: Snippet[]
}

export type UserConfig = FaviconConfig | (() => FaviconConfig | Promise<FaviconConfig>)

/** Identity helper that gives `favicon.config.ts` type checking and editor completion. */
export function defineConfig<const T extends UserConfig>(config: T): T {
  return config
}

export interface GenerateOptions {
  /** Directory that relative paths in `config` are resolved against. @default process.cwd() */
  cwd?: string
}

/**
 * Generates favicons programmatically. Config files are not read; `config` is
 * the complete configuration.
 */
export async function generate(config: FaviconConfig, options: GenerateOptions = {}): Promise<void> {
  const child = spawn(resolveBinary(), ['--config', '-'], {
    cwd: options.cwd,
    env: binaryEnv(),
    stdio: ['pipe', 'inherit', 'inherit'],
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
