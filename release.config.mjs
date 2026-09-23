// semantic-release, run by .github/workflows/release.yml. The workflow builds the
// binaries and packs the npm packages first; this config versions, publishes
// and commits the result.

/** @type {import('semantic-release').GlobalConfig} */
export default {
  branches: ['main', { name: 'next', prerelease: true }],
  plugins: [
    ['@semantic-release/commit-analyzer', {
      preset: 'conventionalcommits',
      // While on 0.x: breaking changes bump the minor version, features and
      // fixes the patch version, so `^0.x.y` ranges only pick up compatible
      // releases. Remove the first two rules to release 1.0.0 with the next
      // breaking change.
      releaseRules: [
        { breaking: true, release: 'minor' },
        { type: 'feat', release: 'patch' },
        // README changes only reach the npm package pages with a release.
        { type: 'docs', scope: 'readme', release: 'patch' },
      ],
    }],
    ['@semantic-release/release-notes-generator', {
      preset: 'conventionalcommits',
      presetConfig: {
        types: [
          { type: 'feat', section: 'Features' },
          { type: 'fix', section: 'Bug Fixes' },
          { type: 'perf', section: 'Performance Improvements' },
          { type: 'revert', section: 'Reverts' },
          { type: 'docs', section: 'Documentation' },
        ],
      },
    }],
    ['@semantic-release/changelog', { changelogTitle: '# Changelog' }],
    ['@semantic-release/exec', {
      // The dry run in the workflow's version job hands the version to the
      // build via NEXT_VERSION_FILE. The real run must arrive at the same
      // version (EXPECTED_VERSION), or the binaries would not match.
      verifyReleaseCmd: 'if [ -n "$NEXT_VERSION_FILE" ]; then echo "${nextRelease.version}" > "$NEXT_VERSION_FILE"; fi'
        + ' && if [ -n "$EXPECTED_VERSION" ] && [ "$EXPECTED_VERSION" != "${nextRelease.version}" ]; then'
        + ' echo "expected version $EXPECTED_VERSION, got ${nextRelease.version}" >&2; exit 1; fi',
      prepareCmd: 'node scripts/set-version.ts ${nextRelease.version}',
      publishCmd: 'node scripts/publish-npm.ts dist-npm ${nextRelease.version} ${nextRelease.channel || "latest"}',
    }],
    ['@semantic-release/git', {
      assets: ['CHANGELOG.md', 'Cargo.toml', 'Cargo.lock', 'packages/*/package.json'],
      message: 'chore(release): ${nextRelease.version} [skip ci]\n\n${nextRelease.notes}',
    }],
    ['@semantic-release/github', {
      assets: [{ path: 'archives/*' }],
      successComment: false,
      failComment: false,
      releasedLabels: false,
    }],
  ],
}
