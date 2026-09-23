// Publishes the tarballs from scripts/pack-npm.ts, the core package first so
// the dependency of the Nuxt module and the unplugin resolves. Versions that
// are already on npm are skipped, so a failed release can be published again.
//
//   node scripts/publish-npm.ts <dir> <version> <dist-tag>
//
// Authenticates with npm trusted publishing (OIDC) in GitHub Actions.
import { execFileSync } from 'node:child_process'
import { resolve } from 'node:path'
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
  // Absolute, because npm reads a relative `dir/file.tgz` as a GitHub `owner/repo` shorthand.
  const tarball = resolve(dir, tarballName(name, version))
  execFileSync('npm', ['publish', tarball, '--access', 'public', '--tag', tag, `--registry=${registry}`], {
    stdio: 'inherit',
  })
}
