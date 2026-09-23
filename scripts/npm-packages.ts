// What the release publishes to npm, shared by the release scripts.

/**
 * Rust targets of the release binaries and the `<os>-<cpu>` directory each gets in the package.
 * Keep in sync with `supportedPlatforms` in packages/favicon-generator/src/binary.ts.
 */
export const TARGETS: Record<string, { os: string, cpu: string }> = {
  'aarch64-apple-darwin': { os: 'darwin', cpu: 'arm64' },
  'x86_64-apple-darwin': { os: 'darwin', cpu: 'x64' },
  'aarch64-unknown-linux-musl': { os: 'linux', cpu: 'arm64' },
  'x86_64-unknown-linux-musl': { os: 'linux', cpu: 'x64' },
  'x86_64-pc-windows-msvc': { os: 'win32', cpu: 'x64' },
}

/** The published packages in dependency order. */
export const PACKAGES = [
  '@denkwerk/favicon-generator',
  '@denkwerk/nuxt-favicon-generator',
  '@denkwerk/unplugin-favicon-generator',
]

/** File name `pnpm pack` and `npm pack` give the tarball of a scoped package. */
export function tarballName(name: string, version: string): string {
  return `${name.slice(1).replace('/', '-')}-${version}.tgz`
}
