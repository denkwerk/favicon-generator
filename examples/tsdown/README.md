# example-tsdown

Shows [`@denkwerk/unplugin-favicon-generator`](../../packages/unplugin-favicon-generator) in a library built with
[tsdown](https://tsdown.dev), e.g. a design system that ships its brand's favicons.

- [`tsdown.config.ts`](tsdown.config.ts) adds the Rolldown plugin (tsdown builds with Rolldown). It writes the icons
  to `dist/favicons/`; `base` is the URL the consuming app serves that directory from.
- [`src/index.ts`](src/index.ts) re-exports the tags from `virtual:favicons`, so consumers can add them to their
  `<head>` without knowing the file names. The exports are typed explicitly, so the generated `.d.mts` does not
  reference `virtual:favicons`, which only exists while the library is built.
- [`package.json`](package.json) exports the files as `example-tsdown/favicons/*` next to the entry.

`pnpm build` runs `tsdown`, and `pnpm test` ([`test/verify.ts`](test/verify.ts)) checks the files in
`dist/favicons/` and the exported tags.
