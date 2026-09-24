// Publishes the tarballs from scripts/pack-npm.ts to npm and to GitHub
// Packages, the core package first so the dependency of the Nuxt module and
// the unplugin resolves. Versions that are already on a registry are skipped,
// so a failed release can be published again.
//
//   node scripts/publish-npm.ts <dir> <version> <dist-tag>
//
// npm authenticates with trusted publishing (OIDC) in GitHub Actions. GitHub
// Packages authenticates with GITHUB_TOKEN (`packages: write`); projects that
// resolve the @denkwerk scope from GitHub Packages install from there.
import { execFileSync } from 'node:child_process'
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join, resolve } from 'node:path'
import { PACKAGES, tarballName } from './npm-packages.ts'

const [dir, version, tag] = process.argv.slice(2)
if (!dir || !version || !tag) {
  console.error('usage: node scripts/publish-npm.ts <dir> <version> <dist-tag>')
  process.exit(1)
}
if (!process.env.GITHUB_TOKEN) {
  console.error('GITHUB_TOKEN is required to publish to GitHub Packages')
  process.exit(1)
}

// Only the GitHub Packages token goes into this file, as a reference that npm
// expands, so the token is never written to disk. npm keeps using the
// trusted-publishing setup of the default user config.
const configDir = mkdtempSync(join(tmpdir(), 'publish-npm-'))
const githubConfig = join(configDir, '.npmrc')
writeFileSync(githubConfig, '//npm.pkg.github.com/:_authToken=${GITHUB_TOKEN}\n')

interface Registry {
  name: string
  url: string
  /** Extra arguments for `npm view` and `npm publish`. */
  args: string[]
}

const registries: Registry[] = [
  { name: 'npm', url: 'https://registry.npmjs.org', args: [] },
  // GitHub Packages does not support provenance statements.
  { name: 'GitHub Packages', url: 'https://npm.pkg.github.com', args: [`--userconfig=${githubConfig}`, '--provenance=false'] },
]

function isPublished(name: string, registry: Registry): boolean {
  try {
    const output = execFileSync('npm', ['view', `${name}@${version}`, 'version', `--registry=${registry.url}`, ...registry.args], {
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore'],
    })
    return output.trim() === version
  } catch {
    return false
  }
}

try {
  for (const registry of registries) {
    for (const name of PACKAGES) {
      if (isPublished(name, registry)) {
        console.log(`${name}@${version} is already on ${registry.name}; skipping`)
        continue
      }
      // Absolute, because npm reads a relative `dir/file.tgz` as a GitHub `owner/repo` shorthand.
      const tarball = resolve(dir, tarballName(name, version))
      console.log(`Publishing ${name}@${version} to ${registry.name}`)
      execFileSync('npm', ['publish', tarball, '--access', 'public', '--tag', tag, `--registry=${registry.url}`, ...registry.args], {
        stdio: 'inherit',
      })
    }
  }
} finally {
  rmSync(configDir, { recursive: true, force: true })
}
