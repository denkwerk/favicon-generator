<p align="center">
  <img src="https://raw.githubusercontent.com/denkwerk/favicon-generator/main/.github/assets/logo.svg" width="80" height="80" alt="">
</p>

<h1 align="center">@denkwerk/nuxt-favicon-generator</h1>

<p align="center">
  A complete favicon set for your Nuxt app, and a manifest or theme color with one option each,<br>generated at build time from one SVG or a Figma component, served and linked for you.
</p>

<p align="center">
  <a href="https://www.npmjs.com/package/@denkwerk/nuxt-favicon-generator"><img src="https://img.shields.io/npm/v/@denkwerk/nuxt-favicon-generator?color=02969c" alt="npm version"></a>
  <a href="https://www.npmjs.com/package/@denkwerk/nuxt-favicon-generator"><img src="https://img.shields.io/npm/dm/@denkwerk/nuxt-favicon-generator?color=02969c" alt="npm downloads"></a>
  <a href="https://github.com/denkwerk/favicon-generator/blob/main/LICENSE"><img src="https://img.shields.io/github/license/denkwerk/favicon-generator?color=02969c" alt="MIT license"></a>
</p>

<p align="center">
  <a href="#setup">Setup</a> · <a href="#config-file">Config file</a> · <a href="#from-figma">Figma</a> · <a href="#options">Options</a> · <a href="https://github.com/denkwerk/favicon-generator">Overview</a>
</p>

A Nuxt module that generates favicons and an Apple touch icon from one SVG, a raster image or a Figma component,
serves the files, and adds the matching `<link>` and `<meta>` tags to every page. Additionally, it can add a web app
manifest, a theme color and Windows tiles. It is built on [`@denkwerk/favicon-generator`](../favicon-generator).

- Nothing is written to `public/`: the files are generated into `node_modules/.cache/favicon-generator` at build time
  and served by Nitro.
- Output is cached until the options or the source image change. A Figma export is reused while the Figma file is
  unchanged, which one small API request checks.
- In `nuxt dev`, editing the source image restarts Nuxt with regenerated icons.

## Setup

```bash
npx nuxt module add @denkwerk/nuxt-favicon-generator
# or: pnpm add -D @denkwerk/nuxt-favicon-generator, then add it to `modules`
```

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ['@denkwerk/nuxt-favicon-generator'],
  favicon: { input: './assets/favicon.svg' },
})
```

That generates and links `favicon.ico`, `favicon.svg`, `favicon-96x96.png` and `apple-touch-icon.png`. Additionally,
you can add a theme color, a web app manifest and more:

```ts
favicon: {
  input: './assets/favicon.svg',
  themeColor: '#02969c',
  manifest: { name: 'My App', shortName: 'App', display: 'standalone' },
},
```

Remove any `favicon.ico` or other icons from `public/`, because they would conflict with the generated ones. The module warns about this.

## Config file

Instead of (or in addition to) the `favicon` options, the module reads a `favicon.config.ts`, the same file the
[CLI](../favicon-generator#configuration) uses:

```ts
// favicon.config.ts
import { defineConfig } from '@denkwerk/favicon-generator'

export default defineConfig({
  input: './assets/favicon.svg',
  themeColor: '#02969c',
  manifest: { name: 'My App' },
})
```

The lookup rules are the CLI's, starting in the Nuxt `rootDir`. The module uses the first of
`favicon.config.{js,ts,mjs,mts,cjs,cts,json}` it finds, then checks parent directories up to the nearest one with a
`package.json` or `.git`. Options in `nuxt.config.ts` take precedence over the file; groups are merged key by key, so
the file can set `manifest.name` and `nuxt.config.ts` `manifest.shortName`. The file's `output`,
`overwrite` and `snippets` are ignored because the module decides where files go, and its `pathPrefix` is relative to
`app.baseURL`. `favicon.config.ts` is type-checked with `nuxt.config.ts`, and in `nuxt dev` editing it restarts Nuxt.
Add `@denkwerk/favicon-generator` to your dependencies so the file can import `defineConfig`.

## From Figma

```ts
favicon: {
  input: 'https://www.figma.com/design/<file-key>/<file-name>?node-id=19938-42',
  // The token comes from FIGMA_TOKEN, e.g. in .env, or a gitignored figmaTokenFile.
},
```

The link must point at a node (`node-id`). The Figma REST API needs a personal access token with the `file_content:read`
scope, which is read from the `FIGMA_TOKEN` env var or `figmaTokenFile`. Keep the token out of your config.

## Options

Besides these module options, every [`@denkwerk/favicon-generator` option](../favicon-generator#options) except
`output`, `overwrite` and `snippets` is supported: `themeColor`, `appleTouchIcon`, `manifest`, `windows`, `legacy` and
the Figma token. Relative paths are resolved against the Nuxt `rootDir`.

| Option | Default | |
| --- | --- | --- |
| `input` | none | Source image or Figma link. Nothing is generated without it. |
| `pathPrefix` | `/` | Path under `app.baseURL` to serve the files from. `/` keeps `/favicon.ico` where browsers look for it. |
| `head` | `true` | Add the `<link>`/`<meta>` tags to every page. |
| `cache` | `true` | Reuse generated files while options and source are unchanged, and a Figma export while the Figma file is unchanged (one small API request per build checks it). `false` generates and exports on every build. |
| `cacheDir` | `node_modules/.cache/favicon-generator` | Where the cache and the generated files are kept. |
| `configFile` | looked up | Path to a specific config file, or `false` to ignore config files. |
| `enabled` | `true` | Turn the module off, e.g. `enabled: process.env.CI !== 'true'`. |

`app.baseURL` is taken into account in all generated URLs and in `manifest.json`.

## Examples

- [`examples/nuxt`](../../examples/nuxt): options in `nuxt.config.ts`
- [`examples/nuxt-config-file`](../../examples/nuxt-config-file): options in `favicon.config.ts`

## License

[MIT](../../LICENSE)
