# example-cli

Shows the `favicon-generator` CLI from [`@denkwerk/favicon-generator`](../../packages/favicon-generator), configured
through a [`favicon.config.ts`](favicon.config.ts).

- [`package.json`](package.json) runs it as `"generate": "favicon-generator"`. Package scripts run in the package
  directory, so the CLI finds `favicon.config.ts` next to `package.json` without any arguments.
- The config exports the icon from Figma when a token is available (`FIGMA_TOKEN` or `.figma-token` at the
  repository root), and otherwise uses [`assets/favicon.svg`](assets/favicon.svg). A config file is plain
  TypeScript, so it can make decisions like that.
- `pnpm typecheck` checks the config against `defineConfig`'s types.

Output goes to `public/favicons` (gitignored). From the repository root, `pnpm turbo run generate --filter=example-cli`
builds the Rust binary and the package first.
