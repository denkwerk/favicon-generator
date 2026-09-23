# favicon-generator

Generates a complete favicon set from an SVG, a raster image or a Figma component. It's a Rust CLI, distributed
on npm as [`@denkwerk/favicon-generator`](packages/favicon-generator), with a Nuxt module,
[`@denkwerk/nuxt-favicon-generator`](packages/nuxt-favicon-generator). See their READMEs for usage and configuration.

## Repository

A [Turborepo](https://turborepo.dev) monorepo that mixes a Cargo workspace (using Turborepo's
[experimental native Rust support](https://turborepo.dev/docs/guides/tools/rust)) with a pnpm workspace:

| Path | Turborepo package | What |
| --- | --- | --- |
| [`crates/favicon-generator`](crates/favicon-generator) | `favicon-generator` | The Rust CLI |
| — | `cargo-workspace` | The Cargo workspace itself (workspace-wide `test`, `lint`, `check`, `format`) |
| [`packages/favicon-generator`](packages/favicon-generator) | `@denkwerk/favicon-generator` | npm package: `defineConfig`, `generate()`, and the CLI launcher |
| [`packages/nuxt-favicon-generator`](packages/nuxt-favicon-generator) | `@denkwerk/nuxt-favicon-generator` | Nuxt module: generates, serves and links the favicons |
| [`examples/cli`](examples/cli) | `example-cli` | The CLI as a package script, configured by `favicon.config.ts` |
| [`examples/cli-figma`](examples/cli-figma) | `example-cli-figma` | The same, exporting the icon from Figma (needs a token) |
| [`examples/typescript`](examples/typescript) | `example-typescript` | The programmatic `generate()` API |
| [`examples/nuxt`](examples/nuxt) | `example-nuxt` | Nuxt app with the module configured in `nuxt.config.ts` |
| [`examples/nuxt-config-file`](examples/nuxt-config-file) | `example-nuxt-config-file` | Nuxt app with the module configured by `favicon.config.ts` |

`@denkwerk/favicon-generator#build` depends on `favicon-generator#build` (see [`turbo.json`](turbo.json)),
so the examples always run against a freshly built binary. Inside the monorepo, the npm package uses
`target/{release,debug}/favicon-generator`. The published package ships the prebuilt binaries of all
platforms in `bin/<os>-<cpu>/` and picks the one for the current platform.

## Development

Requires Node.js ≥ 22.18, pnpm and a Rust toolchain.

```bash
pnpm install
pnpm turbo run build lint test typecheck   # everything, including building and checking the Nuxt example
pnpm turbo run generate                    # run the CLI examples and the TypeScript example (cli-figma needs a Figma token)
pnpm --filter example-nuxt dev             # the Nuxt example in dev mode
pnpm turbo run format                      # cargo fmt
```

To test the Figma export locally, put a personal access token into `.figma-token` (gitignored). In CI, the Figma
example runs when a `FIGMA_TOKEN` repository secret is set.

## Releasing

Releases are automatic: every push to `main` runs the [release workflow](.github/workflows/release.yml), and
[semantic-release](https://semantic-release.gitbook.io) decides from the [Conventional Commits](https://www.conventionalcommits.org)
since the last tag whether to release and which version:

| Commit | While on 0.x | Example |
|---|---|---|
| `fix:`, `perf:`, `feat:` | patch, `0.1.0` → `0.1.1` | `feat(nuxt): add a themeColor option` |
| breaking (`feat!:`, `BREAKING CHANGE:` footer) | minor, `0.1.0` → `0.2.0` | `feat(cli)!: rename --out to --output` |
| `chore:`, `docs:`, `ci:`, `test:`, `refactor:`, … | no release | |

PRs are squash-merged, so their title is the commit message; the [PR title check](.github/workflows/pr-title.yml)
makes sure it is a Conventional Commit. To leave 0.x, remove the `releaseRules` in
[`release.config.mjs`](release.config.mjs): the next breaking change then releases 1.0.0.

For a release, the workflow

- runs CI and builds the binaries for macOS (arm64, x64), Linux (arm64, x64, static musl) and Windows (x64);
- publishes `@denkwerk/favicon-generator` (with all binaries) and `@denkwerk/nuxt-favicon-generator` to npm;
- commits the version bump and [`CHANGELOG.md`](CHANGELOG.md) (`chore(release): x.y.z`), tags it and creates a
  GitHub release with the release notes and the archived binaries.

Pushes to a `next` branch publish prereleases (`0.2.0-next.1`) to the `next` dist-tag.

Running the workflow manually (*Actions → Release → Run workflow*) is a dry run: it builds and packs everything
but publishes nothing. With a tag (`v0.2.0`), it publishes that existing release to npm again, e.g. after a failed
npm publish; versions already on npm are skipped.

npm authentication uses [trusted publishing](https://docs.npmjs.com/trusted-publishers): npm accepts publishes of
both packages only from `release.yml` in this repository, running in the `npm` GitHub environment. There is no npm
token, and the packages do not accept one.

## License

[MIT](LICENSE)
