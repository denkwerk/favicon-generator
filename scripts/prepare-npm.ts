// Copies the release binaries into @denkwerk/favicon-generator, which ships all
// platforms in one package. scripts/pack-npm.ts then packs it.
//
//   node scripts/prepare-npm.ts <artifacts-dir>
//
// <artifacts-dir>/<rust-target>/favicon-generator[.exe]
//   -> packages/favicon-generator/bin/<os>-<cpu>/favicon-generator[.exe]
import { chmodSync, copyFileSync, existsSync, mkdirSync, rmSync } from 'node:fs'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { TARGETS } from './npm-packages.ts'

const artifacts = process.argv[2]
if (!artifacts) {
  console.error('usage: node scripts/prepare-npm.ts <artifacts-dir>')
  process.exit(1)
}

const root = fileURLToPath(new URL('..', import.meta.url))
const mainDir = join(root, 'packages/favicon-generator')

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
