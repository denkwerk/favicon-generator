# favicon-generator

Generates a complete favicon set from an SVG, a raster image or a Figma component. It's a Rust CLI, distributed
on npm as [`@denkwerk/favicon-generator`](packages/favicon-generator). See that README for usage and configuration.

## Repository

A [Turborepo](https://turborepo.dev) monorepo that mixes a Cargo workspace (using Turborepo's
[experimental native Rust support](https://turborepo.dev/docs/guides/tools/rust)) with a pnpm workspace:

| Path | Turborepo package | What |
| --- | --- | --- |
| [`crates/favicon-generator`](crates/favicon-generator) | `favicon-generator` | The Rust CLI |
| — | `cargo-workspace` | The Cargo workspace itself (workspace-wide `test`, `lint`, `check`, `format`) |
| [`packages/favicon-generator`](packages/favicon-generator) | `@denkwerk/favicon-generator` | npm package: `defineConfig`, `generate()`, and the CLI launcher |
| [`demo/typescript`](demo/typescript) | `demo-typescript` | Example TypeScript project using a `favicon.config.ts` |

`@denkwerk/favicon-generator#build` depends on `favicon-generator#build` (see [`turbo.json`](turbo.json)),
so the demo always runs against a freshly built binary. Inside the monorepo, the npm package uses
`target/{release,debug}/favicon-generator`. Once published, it uses the binary from the platform package
`@denkwerk/favicon-generator-<os>-<cpu>`.

## Development

Requires Node.js ≥ 22.18, pnpm and a Rust toolchain.

```bash
pnpm install
pnpm turbo run build lint test typecheck   # everything
pnpm turbo run generate                    # run the demo (uses Figma if a token is available)
pnpm turbo run format                      # cargo fmt
```

To test the Figma export locally, put a personal access token into `.figma-token` (gitignored).

## Releasing

1. `pnpm set-version 1.2.3` updates the crate, `Cargo.lock` and the npm package. Commit the change.
2. Push a tag: `git tag v1.2.3 && git push origin v1.2.3`.

The [release workflow](.github/workflows/release.yml) then:

- builds the binaries for macOS (arm64, x64), Linux (arm64, x64, static musl) and Windows (x64);
- publishes the platform packages and `@denkwerk/favicon-generator` to npm (prereleases such as
  `1.2.3-beta.1` go to the `next` dist-tag);
- creates a GitHub release with the archived binaries.

Running the workflow manually (*Actions → Release → Run workflow*) does a dry run: it builds and packs
everything but publishes nothing.

npm authentication uses [trusted publishing](https://docs.npmjs.com/trusted-publishers) when it is configured for
the packages. Otherwise it uses an `NPM_TOKEN` repository secret, which is needed for the very first publish,
because trusted publishers can only be set up once a package exists.
