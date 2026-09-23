import { existsSync, statSync } from 'node:fs'
import { createRequire } from 'node:module'
import { fileURLToPath } from 'node:url'

const require = createRequire(import.meta.url)
const executable = process.platform === 'win32' ? 'favicon-generator.exe' : 'favicon-generator'

/** npm package that ships the prebuilt binary for the current platform. */
export const platformPackage = `@denkwerk/favicon-generator-${process.platform}-${process.arch}`

// Windows on Arm runs the x64 binary through emulation.
const platformPackages = process.platform === 'win32' && process.arch === 'arm64'
  ? [platformPackage, '@denkwerk/favicon-generator-win32-x64']
  : [platformPackage]

/**
 * Locates the native binary, in order:
 * 1. `FAVICON_GENERATOR_BINARY`
 * 2. the platform package installed as an optional dependency
 * 3. a `cargo build` output when running inside the monorepo
 */
export function resolveBinary(): string {
  const fromEnv = process.env.FAVICON_GENERATOR_BINARY
  if (fromEnv) {
    return fromEnv
  }

  for (const name of platformPackages) {
    try {
      return require.resolve(`${name}/bin/${executable}`)
    } catch {
      // Not installed; try the next option.
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
    `No favicon-generator binary found for ${process.platform}-${process.arch}. `
    + `It is installed through the optional dependency ${platformPackage}; make sure optional dependencies `
    + `are not disabled (--no-optional / --omit=optional), or set FAVICON_GENERATOR_BINARY.`,
  )
}

/** Environment for the binary: JS/TS configs are evaluated with the current Node.js. */
export function binaryEnv(): NodeJS.ProcessEnv {
  return { ...process.env, FAVICON_GENERATOR_NODE: process.execPath }
}
