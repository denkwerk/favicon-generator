# example-typescript

Shows how to use `@denkwerk/favicon-generator` in a TypeScript project.

- [`favicon.config.ts`](favicon.config.ts) is picked up automatically by `pnpm generate` (the `favicon-generator` CLI).
  It exports the icon from Figma when a token is available (`FIGMA_TOKEN` or `.figma-token` at the repository root),
  and otherwise uses [`assets/favicon.svg`](assets/favicon.svg).
- [`scripts/generate.ts`](scripts/generate.ts) uses the programmatic `generate()` API (`pnpm generate:api`).
- `pnpm typecheck` type-checks both, including the config against `defineConfig`'s types.

Output goes to `public/` (gitignored). From the repository root, run `pnpm turbo run generate`: it builds the Rust
binary and the npm package first.
