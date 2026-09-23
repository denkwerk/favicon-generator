// Copies the release binaries into @denkwerk/favicon-generator, which ships all
// platforms in one package, and pins the workspace dependency of the Nuxt module
// to the released version.
//
//   node scripts/prepare-npm.ts <artifacts-dir>
//
// <artifacts-dir>/<rust-target>/favicon-generator[.exe]
//   -> packages/favicon-generator/bin/<os>-<cpu>/favicon-generator[.exe]
// Prints the package directories to publish in dependency order.
import { chmodSync, copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'

// Keep in sync with `supportedPlatforms` in packages/favicon-generator/src/binary.ts.
export const TARGETS: Record<string, { os: string, cpu: string }> = {
  'aarch64-apple-darwin': { os: 'darwin', cpu: 'arm64' },
  'x86_64-apple-darwin': { os: 'darwin', cpu: 'x64' },
  'aarch64-unknown-linux-musl': { os: 'linux', cpu: 'arm64' },
  'x86_64-unknown-linux-musl': { os: 'linux', cpu: 'x64' },
  'x86_64-pc-windows-msvc': { os: 'win32', cpu: 'x64' },
}

const artifacts = process.argv[2]
if (!artifacts) {
  console.error('usage: node scripts/prepare-npm.ts <artifacts-dir>')
  process.exit(1)
}

const root = fileURLToPath(new URL('..', import.meta.url))
const mainDir = join(root, 'packages/favicon-generator')
const main = JSON.parse(readFileSync(join(mainDir, 'package.json'), 'utf8'))

for (const [target, { os, cpu }] of Object.entries(TARGETS)) {
  const executable = os === 'win32' ? 'favicon-generator.exe' : 'favicon-generator'
  const binary = join(artifacts, target, executable)
  if (!existsSync(binary)) {
    throw new Error(`missing binary for ${target}: ${binary}`)
  }

  const dir = join(mainDir, 'bin', `${os}-${cpu}`)
  rmSync(dir, { recursive: true, force: true })
  mkdirSync(dir, { recursive: true })
  copyFileSync(binary, join(dir, executable))
  chmodSync(join(dir, executable), 0o755)
}

// npm publish does not understand pnpm's `workspace:` protocol.
const nuxtDir = join(root, 'packages/nuxt-favicon-generator')
const nuxtPath = join(nuxtDir, 'package.json')
const nuxt = JSON.parse(readFileSync(nuxtPath, 'utf8'))
for (const [name, range] of Object.entries<string>(nuxt.dependencies)) {
  if (range.startsWith('workspace:')) {
    nuxt.dependencies[name] = `^${main.version}`
  }
}
writeFileSync(nuxtPath, `${JSON.stringify(nuxt, null, 2)}\n`)

console.log([mainDir, nuxtDir].join('\n'))
