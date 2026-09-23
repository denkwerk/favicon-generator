# example-vite-config-file

Shows [`@denkwerk/unplugin-favicon-generator`](../../packages/unplugin-favicon-generator) in a Vite app, configured
through a [`favicon.config.ts`](favicon.config.ts) instead of plugin options in [`vite.config.ts`](vite.config.ts).

The plugin looks the config file up with the same rules as the CLI: the first of
`favicon.config.{js,ts,mjs,mts,cjs,cts,json}` in Vite's `root`, then its parents up to the nearest directory with a
`package.json` or `.git`. Options passed to `favicons()` still take precedence. In `vite dev`, editing the config
file regenerates the icons and reloads the page.

`pnpm build` runs `vite build`, and `pnpm test` ([`test/verify.ts`](test/verify.ts)) checks that the files, head tags
and manifest use the values from the config file.

```bash
pnpm dev        # http://localhost:5173, edit favicon.config.ts to see the icons regenerate
pnpm build && pnpm preview
```
