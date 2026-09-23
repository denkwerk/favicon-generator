# example-typescript

Shows the programmatic API of [`@denkwerk/favicon-generator`](../../packages/favicon-generator):
[`scripts/generate.ts`](scripts/generate.ts) calls `generate()` with the complete config as an object, so no
config file is involved. Use this when favicon generation is one step of your own build script.

`pnpm generate` writes to `public/favicons` (gitignored), and `pnpm typecheck` type-checks the script. For the CLI with a
config file, see [`examples/cli`](../cli).
