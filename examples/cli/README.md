# example-cli

Shows the `favicon-generator` CLI from [`@denkwerk/favicon-generator`](../../packages/favicon-generator), configured
through a [`favicon.config.ts`](favicon.config.ts) that generates the icons from [`assets/favicon.svg`](assets/favicon.svg).

- [`package.json`](package.json) runs it as `"generate": "favicon-generator"`. Package scripts run in the package
  directory, so the CLI finds `favicon.config.ts` next to `package.json` without any arguments.
- `pnpm typecheck` checks the config against `defineConfig`'s types.

Output goes to `public/favicons` (gitignored). From the repository root, `pnpm turbo run generate --filter=example-cli`
builds the Rust binary and the package first. To export the icon from Figma instead, see [`examples/cli-figma`](../cli-figma).
