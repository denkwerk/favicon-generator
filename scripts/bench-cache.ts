// Measures what the cache saves: runs the release binary without a cache, with
// an empty cache and with a warm cache, and prints the median times.
//
//   cargo build --release && node scripts/bench-cache.ts [runs]
//
// Set FIGMA_TOKEN (or put it in .env.local) to include a Figma export; it
// makes 6 API requests per run.

import { spawnSync } from 'node:child_process'
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { crc32, deflateSync } from 'node:zlib'

const repo = new URL('..', import.meta.url).pathname
const binary = join(repo, 'target/release', process.platform === 'win32' ? 'favicon-generator.exe' : 'favicon-generator')
const runs = Number(process.argv[2] ?? 5)
const FIGMA_LINK = 'https://www.figma.com/design/77SgSAGXbDv1Eye6htYdCG/ONE---Assets-Library-NEW?node-id=19938-42'
const FULL = ['--legacy', '--manifest', '--maskable', '--windows', '--name', 'Bench']

if (!existsSync(binary)) {
  throw new Error(`${binary} not found; run cargo build --release first`)
}
const envFile = join(repo, '.env.local')
if (!process.env.FIGMA_TOKEN && existsSync(envFile)) {
  process.loadEnvFile(envFile)
}

const dir = mkdtempSync(join(tmpdir(), 'favicon-generator-bench-'))
writeFileSync(join(dir, 'package.json'), '{}')
copyFileSync(join(repo, 'crates/favicon-generator/tests/fixtures/logo.svg'), join(dir, 'logo.svg'))
writeFileSync(join(dir, 'photo.png'), gradientPng(2048))

interface Scenario {
  name: string
  args: string[]
  runs?: number
}
const scenarios: Scenario[] = [
  { name: 'SVG, default options', args: ['logo.svg'] },
  { name: 'SVG, all options (32 files)', args: ['logo.svg', ...FULL] },
  { name: 'PNG 2048×2048, all options', args: ['photo.png', ...FULL] },
]
if (process.env.FIGMA_TOKEN) {
  scenarios.push({ name: 'Figma node, all options', args: [FIGMA_LINK, ...FULL], runs: Math.min(runs, 3) })
} else {
  console.warn('FIGMA_TOKEN is not set; skipping the Figma export.')
}

let cacheDirs = 0
function time(args: string[], cache: string | null): number {
  const start = performance.now()
  const result = spawnSync(binary, [...args, '-o', 'out', '-y', ...(cache ? ['--cache-dir', cache] : ['--no-cache'])], {
    cwd: dir,
    encoding: 'utf8',
  })
  const elapsed = performance.now() - start
  if (result.status !== 0) {
    throw new Error(`favicon-generator ${args.join(' ')} failed:\n${result.stderr}`)
  }
  return elapsed
}

function median(count: number, run: () => number): number {
  const times = Array.from({ length: count }, run).sort((a, b) => a - b)
  return times[Math.floor(count / 2)]!
}

const ms = (value: number) => `${value < 10 ? value.toFixed(1) : Math.round(value)} ms`
const rows = scenarios.map(({ name, args, runs: count = runs }) => {
  // Warms up the OS file cache and the font database.
  time(args, null)
  const uncached = median(count, () => time(args, null))
  const cold = median(count, () => time(args, join(dir, `cache-${cacheDirs++}`)))
  const warmDir = join(dir, 'cache-warm')
  rmSync(warmDir, { recursive: true, force: true })
  mkdirSync(warmDir)
  time(args, warmDir)
  const warm = median(count, () => time(args, warmDir))
  return [name, ms(uncached), ms(cold), ms(warm), `${(uncached / warm).toFixed(1)}×`]
})

console.log(`\nMedian of ${runs} runs (Figma: ${Math.min(runs, 3)}), ${process.platform}-${process.arch}\n`)
const header = ['Input', 'No cache', 'Empty cache', 'Cache hit', 'Speedup']
const table = [header, header.map(() => '---'), ...rows]
for (const row of table) {
  console.log(`| ${row.join(' | ')} |`)
}
rmSync(dir, { recursive: true, force: true })

/** An opaque RGB gradient as PNG, so that there is a large raster input without a fixture. */
function gradientPng(size: number): Buffer {
  const rows = Buffer.alloc(size * (size * 3 + 1))
  for (let y = 0; y < size; y++) {
    const row = y * (size * 3 + 1)
    for (let x = 0; x < size; x++) {
      rows.set([(x * 255) / size, (y * 255) / size, ((x + y) * 127) / size], row + 1 + x * 3)
    }
  }
  const header = Buffer.alloc(13)
  header.writeUInt32BE(size, 0)
  header.writeUInt32BE(size, 4)
  header.set([8, 2, 0, 0, 0], 8)
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk('IHDR', header),
    chunk('IDAT', deflateSync(rows)),
    chunk('IEND', Buffer.alloc(0)),
  ])
}

function chunk(type: string, data: Buffer): Buffer {
  const length = Buffer.alloc(4)
  length.writeUInt32BE(data.length)
  const body = Buffer.concat([Buffer.from(type, 'ascii'), data])
  const crc = Buffer.alloc(4)
  crc.writeUInt32BE(crc32(body))
  return Buffer.concat([length, body, crc])
}
