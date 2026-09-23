# example-vite

Shows [`@denkwerk/unplugin-favicon-generator`](../../packages/unplugin-favicon-generator) in a Vite app.

- [`vite.config.ts`](vite.config.ts) adds the plugin to generate the icons from [`assets/favicon.svg`](assets/favicon.svg).
  The `<head>` tags are injected into [`index.html`](index.html).
- [`src/main.ts`](src/main.ts) imports the same tags from `virtual:favicons` and shows them.
- `pnpm build` runs `vite build`, and `pnpm test` ([`test/verify.ts`](test/verify.ts)) checks that the files and
  head tags are in `dist`.

```bash
pnpm dev        # http://localhost:5173, edit assets/favicon.svg to see the icons regenerate
pnpm build && pnpm preview
```

From the repository root, `pnpm turbo run test --filter=example-vite` builds the Rust binary and the packages first.
