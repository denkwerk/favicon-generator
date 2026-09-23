import { existsSync, statSync } from 'node:fs'
import { fileURLToPath } from 'node:url'

const executable = process.platform === 'win32' ? 'favicon-generator.exe' : 'favicon-generator'

/** Platforms with a prebuilt binary in `bin/<os>-<cpu>/`, see scripts/npm-packages.ts. */
const supportedPlatforms = ['darwin-arm64', 'darwin-x64', 'linux-arm64', 'linux-x64', 'win32-x64']

const platform = `${process.platform}-${process.arch}`

// Windows on Arm runs the x64 binary through emulation.
const platforms = platform === 'win32-arm64' ? [platform, 'win32-x64'] : [platform]

/**
 * Locates the native binary, in order:
 * 1. `FAVICON_GENERATOR_BINARY`
 * 2. the prebuilt binary for the current platform shipped in this package
 * 3. a `cargo build` output when running inside the monorepo
 */
export function resolveBinary(): string {
  const fromEnv = process.env.FAVICON_GENERATOR_BINARY
  if (fromEnv) {
    return fromEnv
  }

  for (const name of platforms) {
    const path = fileURLToPath(new URL(`../bin/${name}/${executable}`, import.meta.url))
    if (existsSync(path)) {
      return path
    }
  }

  const newest = ['release', 'debug']
    .map((profile) => fileURLToPath(new URL(`../../../target/${profile}/${executable}`, import.meta.url)))
    .filter((path) => existsSync(path))
    .sort((a, b) => statSync(b).mtimeMs - statSync(a).mtimeMs)[0]
  if (newest) {
    return newest
  }

  throw new Error(
    `No favicon-generator binary found for ${platform}. Prebuilt binaries are available for `
    + `${supportedPlatforms.join(', ')}; on other platforms, build the binary with cargo and set FAVICON_GENERATOR_BINARY.`,
  )
}

/** Environment for the binary: JS/TS configs are evaluated with the current Node.js. */
export function binaryEnv(): NodeJS.ProcessEnv {
  return { ...process.env, FAVICON_GENERATOR_NODE: process.execPath }
}
