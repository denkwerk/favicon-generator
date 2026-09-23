# example-nuxt-config-file

Shows [`@denkwerk/nuxt-favicon-generator`](../../packages/nuxt-favicon-generator) configured through a
[`favicon.config.ts`](favicon.config.ts) instead of `favicon` options in [`nuxt.config.ts`](nuxt.config.ts).

The module looks the config file up with the same rules as the CLI: the first of
`favicon.config.{js,ts,mjs,mts,cjs,cts,json}` in the Nuxt `rootDir`, then its parents up to the nearest
directory with a `package.json` or `.git`. Options set in `nuxt.config.ts` still take precedence. In `nuxt dev`,
editing the config file restarts Nuxt with regenerated icons.

`pnpm build` runs `nuxt generate`, and `pnpm test` ([`test/verify.ts`](test/verify.ts)) checks that the files,
head tags and manifest use the values from the config file.
