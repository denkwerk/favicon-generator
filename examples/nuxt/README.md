# example-nuxt

Shows [`@denkwerk/nuxt-favicon-generator`](../../packages/nuxt-favicon-generator) in a Nuxt app.

- [`nuxt.config.ts`](nuxt.config.ts) configures the module to generate the icons from [`assets/favicon.svg`](assets/favicon.svg).
- [`app/app.vue`](app/app.vue) shows the generated icons. The `<head>` tags are added by the module.
- `pnpm build` runs `nuxt generate`, and `pnpm test` ([`test/verify.ts`](test/verify.ts)) checks that the files and
  head tags are in the static output.

```bash
pnpm dev        # http://localhost:3000, edit assets/favicon.svg to see the icons regenerate
pnpm build && pnpm preview
```

From the repository root, `pnpm turbo run test --filter=example-nuxt` builds the Rust binary and the packages first.
