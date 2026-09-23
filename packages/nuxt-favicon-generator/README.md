# @denkwerk/nuxt-favicon-generator

A Nuxt module that generates a complete favicon set (PNG sizes, touch icons, `favicon.ico`, web app manifest and
`browserconfig.xml`) from one SVG, a raster image or a Figma component, serves the files, and adds the matching
`<link>` and `<meta>` tags to every page. It is built on [`@denkwerk/favicon-generator`](../favicon-generator).

- Nothing is written to `public/`: the files are generated into `node_modules/.cache` at build time and served by Nitro.
- Output is cached until the options or the source image change, so Figma is only called when something changed.
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
  favicon: {
    input: './assets/favicon.svg',
    appName: 'My App',
    themeColor: '#02969c',
  },
})
```

Remove any `favicon.ico` or other icons from `public/`, because they would conflict with the generated ones. The module warns about this.

## From Figma

```ts
favicon: {
  input: 'https://www.figma.com/design/77SgSAGXbDv1Eye6htYdCG/ONE---Assets-Library-NEW?node-id=19938-42',
  // Token from FIGMA_TOKEN, or:
  figmaTokenFile: '.figma-token',
},
```

The link must point at a node (`node-id`). The Figma REST API needs a personal access token with the `file_content:read`
scope, which is read from the `FIGMA_TOKEN` env var or `figmaTokenFile`. Keep the token out of your config.

## Options

Besides these module options, every [`@denkwerk/favicon-generator` option](../favicon-generator#options) except
`output`, `overwrite` and `snippets` is supported (`appName`, `themeColor`, `backgroundColor`, `manifestCrossorigin`, …).
Relative paths are resolved against the Nuxt `rootDir`.

| Option | Default | |
| --- | --- | --- |
| `input` | none | Source image or Figma link. Nothing is generated without it. |
| `pathPrefix` | `/` | Path under `app.baseURL` to serve the files from. `/` keeps `/favicon.ico` where browsers look for it. |
| `head` | `true` | Add the `<link>`/`<meta>` tags to every page. |
| `cache` | `true` | Reuse generated files while options and source are unchanged. Set to `false` to re-export from Figma on every build. |
| `enabled` | `true` | Turn the module off, e.g. `enabled: process.env.CI !== 'true'`. |

`app.baseURL` is taken into account in all generated URLs and in `manifest.json`.

## Example

See [`examples/nuxt`](../../examples/nuxt).
