// Publishes the tarballs from scripts/pack-npm.ts, the core package first so
// the Nuxt module's dependency resolves. Versions that are already on npm are
// skipped, so a failed release can be published again.
//
//   node scripts/publish-npm.ts <dir> <version> <dist-tag>
//
// Authenticates with npm trusted publishing (OIDC) in GitHub Actions.
import { execFileSync } from 'node:child_process'
import { join } from 'node:path'
import { PACKAGES, tarballName } from './npm-packages.ts'

const [dir, version, tag] = process.argv.slice(2)
if (!dir || !version || !tag) {
  console.error('usage: node scripts/publish-npm.ts <dir> <version> <dist-tag>')
  process.exit(1)
}

const registry = 'https://registry.npmjs.org'

function isPublished(name: string): boolean {
  try {
    const output = execFileSync('npm', ['view', `${name}@${version}`, 'version', `--registry=${registry}`], {
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore'],
    })
    return output.trim() === version
  } catch {
    return false
  }
}

for (const name of PACKAGES) {
  if (isPublished(name)) {
    console.log(`${name}@${version} is already published; skipping`)
    continue
  }
  execFileSync('npm', ['publish', join(dir, tarballName(name, version)), '--access', 'public', '--tag', tag, `--registry=${registry}`], {
    stdio: 'inherit',
  })
}
