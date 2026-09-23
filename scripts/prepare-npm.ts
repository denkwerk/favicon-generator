// Creates one npm package per platform from the release binaries and wires
// them into @denkwerk/favicon-generator as optional dependencies.
//
//   node scripts/prepare-npm.ts <artifacts-dir>
//
// <artifacts-dir>/<rust-target>/favicon-generator[.exe] -> npm/<os>-<cpu>/
// Prints the package directories to publish, platform packages first.
import { chmodSync, copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'

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
const mainPath = join(root, 'packages/favicon-generator/package.json')
const main = JSON.parse(readFileSync(mainPath, 'utf8'))
const outDir = join(root, 'npm')
rmSync(outDir, { recursive: true, force: true })

const optionalDependencies: Record<string, string> = {}
const dirs: string[] = []
for (const [target, { os, cpu }] of Object.entries(TARGETS)) {
  const executable = os === 'win32' ? 'favicon-generator.exe' : 'favicon-generator'
  const binary = join(artifacts, target, executable)
  if (!existsSync(binary)) {
    throw new Error(`missing binary for ${target}: ${binary}`)
  }

  const name = `${main.name}-${os}-${cpu}`
  const dir = join(outDir, `${os}-${cpu}`)
  mkdirSync(join(dir, 'bin'), { recursive: true })
  copyFileSync(binary, join(dir, 'bin', executable))
  chmodSync(join(dir, 'bin', executable), 0o755)
  writeFileSync(join(dir, 'package.json'), `${JSON.stringify({
    name,
    version: main.version,
    description: `The ${os}-${cpu} binary for ${main.name}`,
    repository: main.repository,
    license: main.license,
    os: [os],
    cpu: [cpu],
    files: ['bin'],
  }, null, 2)}\n`)
  writeFileSync(join(dir, 'README.md'), `# ${name}\n\nThe ${os}-${cpu} binary for [\`${main.name}\`](https://www.npmjs.com/package/${main.name}). Install that package instead.\n`)

  optionalDependencies[name] = main.version
  dirs.push(dir)
}

main.optionalDependencies = optionalDependencies
writeFileSync(mainPath, `${JSON.stringify(main, null, 2)}\n`)
dirs.push(join(root, 'packages/favicon-generator'))

console.log(dirs.join('\n'))
