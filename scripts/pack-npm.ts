// Packs the npm packages and checks the tarballs before they are published.
//
// The core package is packed with npm, which keeps the executable bit of the
// binaries (pnpm resets it to 644 for anything not listed in `bin`). The Nuxt
// module is packed with pnpm, which replaces its `workspace:` range with the
// released version.
//
//   node scripts/pack-npm.ts <out-dir> <version>
//
// Run after scripts/set-version.ts, the TypeScript builds and scripts/prepare-npm.ts.
import { execFileSync } from 'node:child_process'
import { mkdirSync, readFileSync, rmSync } from 'node:fs'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { gunzipSync } from 'node:zlib'
import { PACKAGES, TARGETS, tarballName } from './npm-packages.ts'

const MAX_SIZE = 20 * 1024 * 1024

interface Entry { mode: number, content: Buffer }

/** Reads the regular files of a .tgz (ustar, as written by npm and pnpm). */
function readTarball(path: string): Map<string, Entry> {
  const tar = gunzipSync(readFileSync(path))
  const field = (offset: number, length: number) => tar.toString('utf8', offset, offset + length).replace(/\0.*$/s, '')
  const entries = new Map<string, Entry>()
  for (let offset = 0; offset + 512 <= tar.length && tar[offset] !== 0; ) {
    const prefix = field(offset + 345, 155)
    const name = (prefix ? `${prefix}/` : '') + field(offset, 100)
    const mode = Number.parseInt(field(offset + 100, 8).trim() || '0', 8)
    const size = Number.parseInt(field(offset + 124, 12).trim() || '0', 8)
    const type = field(offset + 156, 1)
    if (type === '0' || type === '') {
      entries.set(name.replace(/^package\//, ''), { mode, content: tar.subarray(offset + 512, offset + 512 + size) })
    }
    offset += 512 + Math.ceil(size / 512) * 512
  }
  return entries
}

function check(path: string, name: string, version: string): void {
  const entries = readTarball(path)
  const manifest = JSON.parse(entries.get('package.json')?.content.toString('utf8') ?? '{}')
  if (manifest.name !== name || manifest.version !== version) {
    throw new Error(`${path}: expected ${name}@${version}, found ${manifest.name}@${manifest.version}`)
  }
  const ranges = { ...manifest.dependencies, ...manifest.optionalDependencies, ...manifest.peerDependencies }
  for (const [dependency, range] of Object.entries<string>(ranges)) {
    if (range.startsWith('workspace:')) {
      throw new Error(`${path}: ${dependency} still has the range ${range}`)
    }
  }

  if (name === '@denkwerk/favicon-generator') {
    for (const { os, cpu } of Object.values(TARGETS)) {
      const binary = `bin/${os}-${cpu}/favicon-generator${os === 'win32' ? '.exe' : ''}`
      const entry = entries.get(binary)
      if (!entry) {
        throw new Error(`${path}: ${binary} is missing`)
      }
      if ((entry.mode & 0o111) === 0) {
        throw new Error(`${path}: ${binary} is not executable (mode ${entry.mode.toString(8)})`)
      }
    }
  }
}

const [out, version] = process.argv.slice(2)
if (!out || !version) {
  console.error('usage: node scripts/pack-npm.ts <out-dir> <version>')
  process.exit(1)
}

const root = fileURLToPath(new URL('..', import.meta.url))
const outDir = join(process.cwd(), out)
rmSync(outDir, { recursive: true, force: true })
mkdirSync(outDir, { recursive: true })

for (const name of PACKAGES) {
  const dir = join(root, 'packages', name.split('/')[1]!)
  const packer = name === '@denkwerk/favicon-generator' ? 'npm' : 'pnpm'
  execFileSync(packer, ['pack', '--pack-destination', outDir], { cwd: dir, stdio: ['ignore', 'ignore', 'inherit'] })
  const path = join(outDir, tarballName(name, version))
  check(path, name, version)
  const size = readFileSync(path).length
  console.log(`${path}: ${(size / 1024 / 1024).toFixed(1)} MB`)
  if (size > MAX_SIZE) {
    throw new Error(`${path} is larger than ${MAX_SIZE / 1024 / 1024} MB`)
  }
}
