# @denkwerk/favicon-generator

Generates a complete favicon set (PNG sizes, touch icons, `favicon.ico`, web app manifest,
`browserconfig.xml` and ready-to-paste `<head>` tags) from one SVG, a raster image, or a
Figma component. The work is done by a native Rust binary; the package ships the prebuilt binaries
for all supported platforms and picks the right one at runtime.

```bash
pnpm add -D @denkwerk/favicon-generator
```

Requires Node.js ≥ 22.18 (for TypeScript config files). Prebuilt binaries cover macOS (arm64, x64),
Linux (arm64, x64) and Windows (x64).

## Configuration

Create a `favicon.config.ts` next to your `package.json`:

```ts
import { defineConfig } from '@denkwerk/favicon-generator'

export default defineConfig({
  input: './assets/favicon.svg',
  output: './public/favicons',
  pathPrefix: '/favicons/',
  appName: 'My App',
  themeColor: '#02969c',
})
```

and run it, e.g. from a package script:

```json
{ "scripts": { "favicons": "favicon-generator" } }
```

### Where the config is looked up

The CLI looks for the first of `favicon.config.{js,ts,mjs,mts,cjs,cts,json}`:

1. in the directory the CLI is run from, then
2. in its parent directories, up to the project root: the nearest directory that contains a `package.json`
   or `.git`. The search never continues past that.

Package scripts (`pnpm favicons`, `npm run favicons`) always run in the package's own directory, so
there it effectively means *next to your `package.json`*. Use `--config <path>` to point at another file,
or `--no-config` to ignore config files.

Relative paths in the config are resolved against the config file's directory. JS/TS configs are
evaluated with Node.js, so they can compute values, read env vars, or export an (async) function.
`defineConfig` reports unknown keys in a config object. For a function, annotate its return type
(`(): FaviconConfig => ({ ... })`) to get the same check.
`favicon.config.json` is read without Node.js.

Flags on the command line take precedence over the config file, e.g. `favicon-generator --app-name Staging`.

### Options

| Option | CLI flag | Default |
| --- | --- | --- |
| `input` | `<INPUT>` | *(required)*: an SVG/PNG/JPEG/WebP file, or a Figma link with `node-id` |
| `output` | `[OUTPUT]` | `favicons` |
| `overwrite` | `-y, --overwrite` | `false` |
| `pathPrefix` | `-p, --path-prefix` | `/` |
| `appName` | `-n, --app-name` | `App` |
| `appShortName` | `--app-short-name` | `appName` |
| `appDescription` | `--app-description` | `appName` |
| `themeColor` | `--theme-color` | `#ffffff` |
| `backgroundColor` | `--background-color` | `#ffffff` (also fills transparent pixels in the apple-touch icons) |
| `tileColor` | `--tile-color` | `backgroundColor` |
| `startUrl` | `--start-url` | `/?source=pwa` |
| `scope` | `--scope` | `/` |
| `display` | `--display` | `standalone` |
| `iconPurpose` | `--icon-purpose` | `any maskable` |
| `manifestCrossorigin` | `--manifest-crossorigin` | none (e.g. `use-credentials`) |
| `figmaToken` | `--figma-token`, `FIGMA_TOKEN` | none |
| `figmaTokenFile` | `--figma-token-file`, `FIGMA_TOKEN_FILE` | none |
| `snippets` | `--snippets` | `['html', 'nuxt']` (also `'json'`; `[]` for none) |

## From Figma

Use a Figma link to a component (it must contain `node-id`) as `input`. The node is exported as SVG through
the [Figma REST API](https://developers.figma.com/docs/rest-api/).

```ts
export default defineConfig({
  input: 'https://www.figma.com/design/77SgSAGXbDv1Eye6htYdCG/ONE---Assets-Library-NEW?node-id=19938-42',
  figmaTokenFile: '.figma-token', // or set FIGMA_TOKEN
})
```

The API needs a personal access token with the `file_content:read` scope (Figma → Settings → Security →
Personal access tokens). Keep it out of version control: use the `FIGMA_TOKEN` env var, or a gitignored token file.

## Programmatic API

```ts
import { generate } from '@denkwerk/favicon-generator'

await generate({ input: 'assets/favicon.svg', output: 'public/favicons' }, { cwd: import.meta.dirname })
```

`generate()` does not read config files: the object you pass is the whole config. To use a config file from
your own tooling, `loadConfig()` finds and evaluates it with the CLI's rules:

```ts
import { loadConfig } from '@denkwerk/favicon-generator'

const { path, config } = await loadConfig({ cwd: process.cwd() }) // path is null if none was found
```

## Output

| File | Notes |
| --- | --- |
| `favicon-{16,32,57,60,70,72,76,96,114,120,128,144,150,152,180,192,310,384,512}.png` | transparent |
| `apple-touch-icon.png`, `apple-touch-icon-{120x120,152x152}.png` (+ `-precomposed`) | opaque |
| `favicon.ico` | 16, 24, 32, 48, 64, 128, 256 |
| `favicon.svg` | copy of an SVG input |
| `manifest.json`, `browserconfig.xml` | web app manifest, Windows tiles |
| `favicon.html` | `<link>`/`<meta>` tags for `<head>` |
| `nuxt-head.ts` | the same tags as `faviconHead`, for `app.head` in `nuxt.config.ts` |
| `favicon-head.json` | the same tags as `{ link, meta }` JSON (only with `snippets: ['json']`) |

For Nuxt, the [`@denkwerk/nuxt-favicon-generator`](../nuxt-favicon-generator) module does all of this for you
at build time: it generates the files, serves them and adds the tags.

## License

[MIT](../../LICENSE)
