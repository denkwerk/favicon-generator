<p align="center">
  <img src="https://raw.githubusercontent.com/denkwerk/favicon-generator/main/.github/assets/logo.svg" width="80" height="80" alt="">
</p>

<h1 align="center">@denkwerk/favicon-generator</h1>

<p align="center">
  One SVG in, every favicon out: a complete favicon set with ready-to-paste <code>&lt;head&gt;</code> tags,<br>and a web app manifest, theme color or Windows tiles with one option each.
</p>

<p align="center">
  <a href="https://www.npmjs.com/package/@denkwerk/favicon-generator"><img src="https://img.shields.io/npm/v/@denkwerk/favicon-generator?color=02969c" alt="npm version"></a>
  <a href="https://www.npmjs.com/package/@denkwerk/favicon-generator"><img src="https://img.shields.io/npm/dm/@denkwerk/favicon-generator?color=02969c" alt="npm downloads"></a>
  <a href="https://github.com/denkwerk/favicon-generator/blob/main/LICENSE"><img src="https://img.shields.io/github/license/denkwerk/favicon-generator?color=02969c" alt="MIT license"></a>
</p>

<p align="center">
  <a href="#usage">Usage</a> · <a href="#what-you-get">What you get</a> · <a href="#configuration">Configuration</a> · <a href="#options">Options</a> · <a href="#figma">Figma</a> · <a href="#nodejs-api">API</a> · <a href="https://github.com/denkwerk/favicon-generator">Overview</a>
</p>

Generates favicons, an Apple touch icon and `<head>` tags from one SVG, a raster image or a Figma component.
Additionally, it can add a web app manifest, a theme color and Windows tiles. The work is done by a native Rust binary; the
package ships prebuilt binaries for macOS (arm64, x64), Linux (arm64, x64) and Windows (x64).

```bash
pnpm add -D @denkwerk/favicon-generator
```

Requires Node.js ≥ 22.18 for TypeScript config files.

## Usage

```bash
favicon-generator -i ./assets/favicon.svg -o ./public/favicons
```

or, with the same options in a `favicon.config.ts` next to your `package.json`, just `favicon-generator`:

```ts
import { defineConfig } from '@denkwerk/favicon-generator'

export default defineConfig({
  input: './assets/favicon.svg',
  output: './public/favicons',
})
```

Flags override the config file. `favicon-generator ./assets/favicon.svg ./public/favicons` is short for `-i … -o …`.

## What you get

One command gives you a complete, modern favicon set: `favicon.ico` (16, 32, 48 px), `favicon.svg` (SVG input only),
`favicon-96x96.png`, `apple-touch-icon.png` (180 px, opaque) and `favicon.html` with these tags:

```html
<link rel="icon" href="/favicon.ico" sizes="32x32">
<link rel="icon" type="image/svg+xml" href="/favicon.svg">
<link rel="icon" type="image/png" sizes="96x96" href="/favicon-96x96.png">
<link rel="apple-touch-icon" href="/apple-touch-icon.png">
```

Additionally, you can add a web app manifest, a theme color, Windows tiles and legacy sizes with one option each,
see [Options](#options).

## Configuration

Options can be given as flags, in a `favicon.config.{ts,mts,cts,js,mjs,cjs,json}`, or to `generate()`; the names are
the same everywhere (flags in kebab-case). The config file is looked up in the current directory, then its parents up
to the nearest `package.json` or `.git`. `--config <path>` picks a file, `--no-config` ignores them.

Relative paths in the config are resolved against its directory. JS/TS configs are evaluated with Node.js, so they can
compute values, read env vars or export an (async) function; `favicon.config.json` is read without Node.js and can
point `"$schema"` at `./node_modules/@denkwerk/favicon-generator/schema.json` for editor support.

`defineConfig` reports unknown keys in a config object. For a function, annotate its return type
(`async (): Promise<FaviconConfig> => ({ ... })`) to get the same check.

## Options

The additions (in bold) are groups: set one to `true` for its defaults or to an object with its options; setting any
of its options or flags adds it too. `false` or `--no-<group>` leaves it out, e.g. the Apple touch icon, which is
included by default.

| Option | Flag | Default / effect |
| --- | --- | --- |
| `input` | `-i, --input`, first argument | *(required)* SVG, PNG, JPEG or WebP, or a Figma link with `node-id` |
| `output` | `-o, --output`, second argument | `favicons` |
| `pathPrefix` | `-p, --path-prefix` | `/`; the URL the files are served from |
| `overwrite` | `-y, --overwrite` | `false` |
| `snippets` | `--snippets` | `['html']`; also `'json'` (`favicon-head.json`) and `'nuxt'` (`nuxt-head.ts`); `[]` for none |
| `themeColor` | `--theme-color` | adds `<meta name="theme-color">` and the manifest's `theme_color` |
| **`appleTouchIcon`** | `--[no-]apple-touch-icon` | included: `apple-touch-icon.png` |
| `appleTouchIcon.background` | `--apple-touch-background` | `#ffffff`; behind transparent pixels |
| **`manifest`** | `--[no-]manifest` | adds `manifest.json`, 192/512 px icons, `<link rel="manifest">` |
| `manifest.name` | `-n, --name` | `name` |
| `manifest.shortName` | `--short-name` | `short_name` |
| `manifest.description` | `--description` | `description` |
| `manifest.startUrl` | `--start-url` | `start_url` |
| `manifest.scope` | `--scope` | `scope` |
| `manifest.display` | `--display` | `display`: `fullscreen`, `standalone`, `minimal-ui`, `browser` |
| `manifest.backgroundColor` | `--background-color` | `background_color`, also behind maskable icons |
| `manifest.maskable` | `--maskable` | adds maskable icons (the image at 60 % on `backgroundColor`), one per icon size |
| `manifest.crossorigin` | `--manifest-crossorigin` | `crossorigin` on the manifest `<link>` |
| `manifest.iconSizes` | `--manifest-icon-sizes` | icon sizes listed in the manifest (default `192,512`) |
| `manifest.iconPurpose` | `--manifest-icon-purpose` | `purpose` of those icons, e.g. `any maskable` |
| **`windows`** | `--[no-]windows` | adds `browserconfig.xml`, tile images, `msapplication-*` tags |
| `windows.tileColor` | `--tile-color` | `msapplication-TileColor` |
| **`legacy`** | `--[no-]legacy` | adds 19 PNG sizes, sized Apple touch icons, a 7-frame `favicon.ico` |
| `figmaToken` | `--figma-token`, `FIGMA_TOKEN` | see [Figma](#figma) |
| `figmaTokenFile` | `--figma-token-file`, `FIGMA_TOKEN_FILE` | see [Figma](#figma) |
| `cache` | `--no-cache` | `true`; see [Cache](#cache) |
| `cacheDir` | `--cache-dir`, `FAVICON_GENERATOR_CACHE_DIR` | `node_modules/.cache/favicon-generator` in the project root, if it has a `node_modules` |

The manifest contains exactly the fields you set; give it a `name` or `shortName` so browsers can offer installing the
app.

```ts
export default defineConfig({
  input: './assets/favicon.svg',
  output: './public/favicons',
  pathPrefix: '/favicons/',
  themeColor: '#02969c',
  manifest: { name: 'My App', shortName: 'App', display: 'standalone' },
  windows: true,
})
```

## Figma

Use a link to a component (it must contain `node-id`) as `input`. The node is exported as SVG through the
[Figma REST API](https://developers.figma.com/docs/rest-api/), which needs a personal access token with the
`file_content:read` scope (Figma → Settings → Security → Personal access tokens). Pass it in the `FIGMA_TOKEN` env var
(e.g. from a gitignored `.env.local`) or a gitignored `figmaTokenFile`, not in the config itself.

```ts
export default defineConfig({
  input: 'https://www.figma.com/design/<file-key>/<file-name>?node-id=19938-42',
  output: './public/favicons',
})
```

## Cache

While the input and the options are unchanged, the generated files are copied from the cache instead of being
rendered again. A Figma export is reused while the Figma file's version is unchanged, which costs one small API
request instead of an export and a download; when that request fails (offline, rate limited, no token), the cached
export is used with a warning. `cache: false` / `--no-cache` turns the cache off, `cacheDir` / `--cache-dir` moves it.
Without a `node_modules` in the project root and without `cacheDir`, nothing is cached.

## Node.js API

```ts
import { generate } from '@denkwerk/favicon-generator'

await generate({ input: 'assets/favicon.svg', output: 'public/favicons', manifest: { name: 'My App' } }, { cwd: import.meta.dirname })
```

`generate()` does not read config files: the object you pass is the whole config. To use a config file from your own
tooling, `loadConfig()` finds and evaluates it with the CLI's rules:

```ts
import { loadConfig } from '@denkwerk/favicon-generator'

const { path, config } = await loadConfig({ cwd: process.cwd() }) // path is null if none was found
```

`mergeConfig(config, overrides)` puts your own options on top the way CLI flags do: `undefined` values are skipped,
and groups such as `manifest` are merged key by key.

For Nuxt, the [`@denkwerk/nuxt-favicon-generator`](https://www.npmjs.com/package/@denkwerk/nuxt-favicon-generator)
module does all of this at build time: it generates the files, serves them and adds the tags. For Vite, Rollup,
Rolldown, webpack and Rspack, [`@denkwerk/unplugin-favicon-generator`](https://www.npmjs.com/package/@denkwerk/unplugin-favicon-generator)
emits them with the build.

## License

[MIT](https://github.com/denkwerk/favicon-generator/blob/main/LICENSE)
