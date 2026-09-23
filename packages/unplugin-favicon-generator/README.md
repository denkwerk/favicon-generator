<p align="center">
  <img src="https://raw.githubusercontent.com/denkwerk/favicon-generator/main/.github/assets/logo.svg" width="80" height="80" alt="">
</p>

<h1 align="center">@denkwerk/unplugin-favicon-generator</h1>

<p align="center">
  A complete favicon set for your Vite, Rollup, Rolldown, webpack or Rspack build,<br>generated from one SVG or a Figma component, emitted with your assets and linked for you.
</p>

<p align="center">
  <a href="https://www.npmjs.com/package/@denkwerk/unplugin-favicon-generator"><img src="https://img.shields.io/npm/v/@denkwerk/unplugin-favicon-generator?color=02969c" alt="npm version"></a>
  <a href="https://www.npmjs.com/package/@denkwerk/unplugin-favicon-generator"><img src="https://img.shields.io/npm/dm/@denkwerk/unplugin-favicon-generator?color=02969c" alt="npm downloads"></a>
  <a href="https://github.com/denkwerk/favicon-generator/blob/main/LICENSE"><img src="https://img.shields.io/github/license/denkwerk/favicon-generator?color=02969c" alt="MIT license"></a>
</p>

<p align="center">
  <a href="#setup">Setup</a> · <a href="#the-tags-in-your-own-html">Tags in your own HTML</a> · <a href="#config-file">Config file</a> · <a href="#options">Options</a> · <a href="https://github.com/denkwerk/favicon-generator">Overview</a>
</p>

A bundler plugin, written once with [unplugin](https://github.com/unjs/unplugin), that generates favicons and an
Apple touch icon from one SVG, a raster image or a Figma component and writes them into your build output.
Additionally, it can add a web app manifest, a theme color and Windows tiles. It is built on
[`@denkwerk/favicon-generator`](../favicon-generator).

- Nothing is written to `public/`: the files are generated into `node_modules/.cache` and emitted as build assets.
- Output is cached until the options or the source image change, so Figma is only called when something changed.
- **Vite**: the tags are injected into every HTML page and follow `base`; the dev server serves the files and
  reloads the page when the image or the config file changes. Server (SSR) builds get no copies of the files.

## Setup

```bash
pnpm add -D @denkwerk/unplugin-favicon-generator
```

<details open>
<summary><b>Vite</b></summary>

```ts
// vite.config.ts
import favicons from '@denkwerk/unplugin-favicon-generator/vite'

export default defineConfig({
  plugins: [favicons({ input: './assets/favicon.svg' })],
})
```

</details>

<details>
<summary><b>Rollup</b></summary>

```ts
// rollup.config.js
import favicons from '@denkwerk/unplugin-favicon-generator/rollup'

export default {
  plugins: [favicons({ input: './assets/favicon.svg' })],
}
```

</details>

<details>
<summary><b>Rolldown, tsdown</b></summary>

```ts
// rolldown.config.js
import favicons from '@denkwerk/unplugin-favicon-generator/rolldown'

export default {
  plugins: [favicons({ input: './assets/favicon.svg' })],
}
```

[tsdown](https://tsdown.dev) builds with Rolldown, so it takes the same plugin in `plugins` of `tsdown.config.ts`.

</details>

<details>
<summary><b>webpack</b></summary>

```js
// webpack.config.js
import favicons from '@denkwerk/unplugin-favicon-generator/webpack'

export default {
  plugins: [favicons({ input: './assets/favicon.svg' })],
}
```

</details>

<details>
<summary><b>Rspack</b></summary>

```js
// rspack.config.js
import favicons from '@denkwerk/unplugin-favicon-generator/rspack'

export default {
  plugins: [favicons({ input: './assets/favicon.svg' })],
}
```

</details>

That's it for Vite. Other bundlers have no HTML to add the tags to, so render them yourself (see below).
Remove any `favicon.ico` or other icons from `public/`, because they would conflict with the generated ones. The Vite
plugin warns about this.

## The tags in your own HTML

The tags are also available as a module, e.g. for an SSR template or a page that is not processed by Vite:

```ts
import { html, link, meta } from 'virtual:favicons' // `~favicons` in webpack and Rspack

html // '<link rel="icon" href="/favicon.ico" sizes="32x32">\n…'
link // [{ rel: 'icon', href: '/favicon.ico', sizes: '32x32' }, …]
meta // [{ name: 'theme-color', content: '#02969c' }, …]
```

webpack and Rspack read `virtual:` as a URL scheme, so import `~favicons` there; it works in the other bundlers
too. For the types, add `@denkwerk/unplugin-favicon-generator/client` to `compilerOptions.types` in your
`tsconfig.json`.

## Config file

Instead of (or in addition to) plugin options, the plugin reads a `favicon.config.ts`, the same file the
[CLI](../favicon-generator#configuration) uses:

```ts
// favicon.config.ts
import { defineConfig } from '@denkwerk/favicon-generator'

export default defineConfig({
  input: './assets/favicon.svg',
  themeColor: '#02969c',
})
```

It is looked up with the CLI's rules, starting in Vite's `root` (or `root`, see below): the first of
`favicon.config.{js,ts,mjs,mts,cjs,cts,json}`, then parent directories up to the nearest one with a `package.json`
or `.git`. Plugin options take precedence over the file, and the options of a group (`manifest`, …) are merged key
by key. The file's `output`, `overwrite` and `snippets` are ignored because the plugin decides where files go.
Add `@denkwerk/favicon-generator` to your dependencies so the file can import `defineConfig`.

## Options

Besides these plugin options, every [`@denkwerk/favicon-generator` option](../favicon-generator#options) except
`output`, `overwrite` and `snippets` is supported (`themeColor`, `manifest`, `windows`, `legacy`, `figmaToken`, …).
Relative paths are resolved against `root`.

| Option | Default | |
| --- | --- | --- |
| `input` | none | Source image or Figma link. Nothing is generated without it. |
| `pathPrefix` | `/` | Directory in the build output, and URL path below `base`, for the files. `/` keeps `/favicon.ico` where browsers look for it. |
| `base` | `/` | URL the build output is served from, e.g. a CDN. Vite uses its own `base`. |
| `root` | Vite's `root`, else `process.cwd()` | Where the config file lookup and relative paths start, and where `node_modules/.cache` is. |
| `inject` | `true` | Vite: add the `<link>`/`<meta>` tags to every HTML page. |
| `cache` | `true` | Reuse generated files while options and source are unchanged. Set to `false` to re-export from Figma on every build. |
| `configFile` | looked up | Path to a specific config file, or `false` to ignore config files. |

## Examples

- [`examples/vite`](../../examples/vite): a Vite app with the tags injected and read from `virtual:favicons`
- [`examples/vite-config-file`](../../examples/vite-config-file): options in `favicon.config.ts`
- [`examples/tsdown`](../../examples/tsdown): a library built with [tsdown](https://tsdown.dev) that ships its favicons and exports their tags

## License

[MIT](../../LICENSE)
