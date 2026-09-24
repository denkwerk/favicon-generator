# Contributing

Thanks for helping out! This file covers the development setup, the tests, how the repository is organized and how
releases work. For using the packages, see the [README](README.md).

## Setup

The tool versions are pinned (see [Tool versions](#tool-versions)). With [mise](https://mise.jdx.dev), one
command installs all of them:

```bash
mise install
```

Without mise, install Node.js from [`mise.toml`](mise.toml) (≥ 22.18 works), pnpm and
[rustup](https://rustup.rs), which picks up [`rust-toolchain.toml`](rust-toolchain.toml).

```bash
pnpm install
pnpm turbo run build lint test typecheck   # everything: crate, npm packages, tests and the Nuxt and Vite examples
pnpm turbo run generate                    # run the CLI and Node.js API examples
pnpm --filter example-nuxt dev             # the Nuxt example in dev mode
pnpm --filter example-vite dev             # the Vite example in dev mode
```

To try the Figma export, copy [`.env.example`](.env.example) to `.env.local` (gitignored) and set `FIGMA_TOKEN`. In CI,
the Figma example runs when a `FIGMA_TOKEN` repository secret is set.

### Tool versions

| Tool | Pinned in | Notes |
| --- | --- | --- |
| Rust, clippy, rustfmt | [`rust-toolchain.toml`](rust-toolchain.toml) | read by rustup, IDEs and mise |
| pnpm | `packageManager` in [`package.json`](package.json) | pnpm switches to that version itself |
| Node.js, zig, cargo-zigbuild | [`mise.toml`](mise.toml), with checksums in [`mise.lock`](mise.lock) | zig and cargo-zigbuild build the static Linux binaries of a release |
| mise itself in CI | `MISE_VERSION` in [`ci.yml`](.github/workflows/ci.yml) and [`release.yml`](.github/workflows/release.yml) | |

CI and the release workflow install the same versions. Dependabot proposes new Rust releases and GitHub Actions;
the tools in `mise.toml` are bumped by hand:

```bash
mise outdated --bump   # newer versions, also across majors
# edit the versions in mise.toml, then update the checksums for all CI platforms:
mise lock --platform linux-x64,linux-arm64,macos-arm64,macos-x64,windows-x64
```

A new Rust release can bring new clippy lints, and CI denies warnings, so bump Rust in a PR of its own. The
`engines` ranges in the `package.json` files are the Node.js versions the packages support, not the development
version.

To build a static Linux binary as a release does: `rustup target add x86_64-unknown-linux-musl`, then
`cargo zigbuild --release --target x86_64-unknown-linux-musl`.

Commit messages and PR titles are [Conventional Commits](https://www.conventionalcommits.org), because they decide
the next release (see [Releasing](#releasing)).

## Tests

| Where | What | Run |
| --- | --- | --- |
| `crates/favicon-generator/src` | unit tests: config parsing and merging, Figma links, the JSON Schema | `cargo test` |
| `crates/favicon-generator/tests` | the binary end to end: files and tags per option ([snapshots](crates/favicon-generator/tests/snapshots)), images, config files, errors, the cache, Figma against a mock server | `cargo test` |
| `packages/favicon-generator/test` | `generate()` and `loadConfig()`; type tests for `defineConfig`; TS types vs. JSON Schema | `pnpm test`, `pnpm typecheck` |
| `packages/nuxt-favicon-generator/test` | option merging; builds and serves [fixture apps](packages/nuxt-favicon-generator/test/fixtures) and checks files and tags | `pnpm test` |
| `packages/unplugin-favicon-generator/test` | real Vite (build, SSR build, dev server), Rollup, Rolldown, webpack and Rspack builds of [fixtures](packages/unplugin-favicon-generator/test/fixtures): emitted files, tags, `virtual:favicons`, config files, regeneration in dev | `pnpm test` |

`pnpm turbo run test` runs all of them, as CI does on Linux, macOS and Windows. On Windows, run the crate's tests
first (`pnpm turbo run test --filter=favicon-generator`), as CI does: `cargo test` relinks
`target/debug/favicon-generator.exe`, which fails while another task is running it.

What the cache saves, measured with the release binary (`FIGMA_TOKEN` adds a real Figma export):

```bash
cargo build --release && node scripts/bench-cache.ts
```

After an intended output change, accept the snapshots with `INSTA_UPDATE=always cargo test`, or review them one by
one with [`cargo insta review`](https://insta.rs/docs/cli/) (`cargo install cargo-insta`).
[`schema.json`](packages/favicon-generator/schema.json) is generated from the Rust config types:
`UPDATE_SCHEMA=1 cargo test` rewrites it.

## Repository layout

A [Turborepo](https://turborepo.dev) monorepo that combines a Cargo workspace (with Turborepo's
[Rust support](https://turborepo.dev/docs/guides/tools/rust)) and a pnpm workspace:

| Path | Turborepo package | What |
| --- | --- | --- |
| [`crates/favicon-generator`](crates/favicon-generator) | `favicon-generator` | The Rust CLI |
| — | `cargo-workspace` | The Cargo workspace itself (workspace-wide `test`, `lint`, `check`, `format`) |
| [`packages/favicon-generator`](packages/favicon-generator) | `@denkwerk/favicon-generator` | npm package: `defineConfig`, `generate()` and the CLI launcher |
| [`packages/nuxt-favicon-generator`](packages/nuxt-favicon-generator) | `@denkwerk/nuxt-favicon-generator` | Nuxt module |
| [`packages/unplugin-favicon-generator`](packages/unplugin-favicon-generator) | `@denkwerk/unplugin-favicon-generator` | Bundler plugin (unplugin) |
| [`examples/*`](examples) | `example-*` | The examples listed in the [README](README.md#packages), run in CI |

`@denkwerk/favicon-generator#build` depends on `favicon-generator#build` (see [`turbo.json`](turbo.json)), so the
examples always run against a freshly built binary from `target/{release,debug}`. The published package ships the
binaries of all platforms in `bin/<os>-<cpu>/` and picks the one for the current platform.

## Releasing

Releases are automatic: every push to `main` runs the [release workflow](.github/workflows/release.yml), and
[semantic-release](https://semantic-release.gitbook.io) decides from the [Conventional Commits](https://www.conventionalcommits.org)
since the last tag whether to release and which version:

| Commit | While on 0.x | Example |
| --- | --- | --- |
| `fix:`, `perf:`, `feat:`, `docs(readme):` | patch, `0.1.0` → `0.1.1` | `feat(nuxt): add a themeColor option` |
| breaking (`feat!:`, `BREAKING CHANGE:` footer) | minor, `0.1.0` → `0.2.0` | `feat(cli)!: rename --out to --output` |
| `chore:`, `docs:`, `ci:`, `test:`, `refactor:`, … | no release | |

`docs(readme):` releases so that README changes reach the npm package pages. PRs are squash-merged, so their title
is the commit message; the [PR title check](.github/workflows/pr-title.yml) makes sure it is a Conventional Commit.
To leave 0.x, remove the `releaseRules` for breaking changes and features in [`release.config.mjs`](release.config.mjs).

A release runs CI, builds the binaries, publishes the packages to npm and GitHub Packages, commits the version bump and
[`CHANGELOG.md`](CHANGELOG.md) (`chore(release): x.y.z`), tags it and creates a GitHub release with the archived
binaries. Pushes to a `next` branch publish prereleases (`0.2.0-next.1`) to the `next` dist-tag.

Running the workflow manually (*Actions → Release → Run workflow*) is a dry run that builds and packs everything.
With a tag (`v0.2.0`), it publishes that existing release again, e.g. after a failed publish; versions already on
a registry are skipped.

npm authentication uses [trusted publishing](https://docs.npmjs.com/trusted-publishers): npm accepts publishes of
the packages only from `release.yml` in this repository, in the `npm` GitHub environment. There is no npm token.
GitHub Packages gets the same tarballs, published with the workflow's `GITHUB_TOKEN` (`packages: write`) and
without provenance, which GitHub Packages does not support. It needs no setup for a new package.

### Adding a package

A new npm package needs a one-time setup before its first release, because trusted publishing can only be configured
for a package that already exists on npm:

1. Add it to `PACKAGES` in [`scripts/npm-packages.ts`](scripts/npm-packages.ts), to
   [`scripts/set-version.ts`](scripts/set-version.ts) and to the build step of the `package` job in
   [`release.yml`](.github/workflows/release.yml).
2. Publish a placeholder `0.0.0` by hand (`npm publish --access public` from a directory with just a `package.json`
   and a README). It can take a few minutes until npm shows the new package.
3. Configure the trusted publisher: on npmjs.com under the package's *Settings → Trusted Publisher* (GitHub Actions,
   repository `denkwerk/favicon-generator`, workflow `release.yml`, environment `npm`), or with
   `npm trust github <package> --file release.yml --repository denkwerk/favicon-generator --environment npm`.

Push only after that: the release publishes the packages one after another, so a package without a trusted publisher
fails the release after the others were published.
