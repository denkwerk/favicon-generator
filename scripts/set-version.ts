// Sets (or with --check, verifies) the version of the crate and the npm package.
//
//   node scripts/set-version.ts 1.2.3
//   node scripts/set-version.ts --check 1.2.3
import { readFileSync, writeFileSync } from 'node:fs'

const args = process.argv.slice(2)
const check = args.includes('--check')
const version = args.find((arg) => !arg.startsWith('--'))?.replace(/^v/, '')

if (!version || !/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/.test(version)) {
  console.error('usage: node scripts/set-version.ts [--check] <semver>')
  process.exit(1)
}

const files: { path: string, pattern: RegExp }[] = [
  // [workspace.package] version, inherited by all crates.
  { path: 'Cargo.toml', pattern: /(\[workspace\.package\]\nversion = ")([^"]+)(")/ },
  { path: 'Cargo.lock', pattern: /(name = "favicon-generator"\nversion = ")([^"]+)(")/ },
  { path: 'packages/favicon-generator/package.json', pattern: /(\n {2}"version": ")([^"]+)(")/ },
]

let mismatches = 0
for (const { path, pattern } of files) {
  const url = new URL(`../${path}`, import.meta.url)
  const content = readFileSync(url, 'utf8')
  const current = pattern.exec(content)?.[2]
  if (!current) {
    throw new Error(`no version found in ${path}`)
  }
  if (check) {
    if (current !== version) {
      console.error(`${path}: ${current} (expected ${version})`)
      mismatches++
    }
  } else if (current !== version) {
    writeFileSync(url, content.replace(pattern, `$1${version}$3`))
    console.log(`${path}: ${current} -> ${version}`)
  }
}

if (mismatches > 0) {
  console.error(`Run \`pnpm set-version ${version}\` and commit before tagging.`)
  process.exit(1)
}
