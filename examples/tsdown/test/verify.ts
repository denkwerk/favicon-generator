// Checks the library build (`tsdown`): the generated files are in
// dist/favicons/, and the bundled entry exports their tags with `base`.
import { existsSync } from 'node:fs'

const output = new URL('../dist/', import.meta.url)
const failures: string[] = []

for (const file of ['favicon.ico', 'favicon.svg', 'favicon-96x96.png', 'favicon-192x192.png', 'apple-touch-icon.png', 'manifest.json']) {
  if (!existsSync(new URL(`favicons/${file}`, output))) {
    failures.push(`missing favicons/${file}`)
  }
}
for (const file of ['index.mjs', 'index.d.mts']) {
  if (!existsSync(new URL(file, output))) {
    failures.push(`missing ${file}`)
  }
}

const { faviconHead, faviconHtml } = await import(new URL('index.mjs', output).href)
for (const tag of [
  { rel: 'icon', type: 'image/svg+xml', href: '/assets/my-library/favicons/favicon.svg' },
  { rel: 'manifest', href: '/assets/my-library/favicons/manifest.json' },
]) {
  if (!faviconHead.link.some((link: Record<string, string>) => JSON.stringify(link) === JSON.stringify(tag))) {
    failures.push(`faviconHead.link lacks ${JSON.stringify(tag)}`)
  }
}
if (!faviconHtml.includes('<meta name="theme-color" content="#02969c">')) {
  failures.push('faviconHtml lacks the theme color')
}

if (failures.length > 0) {
  console.error(failures.join('\n'))
  process.exit(1)
}
console.log('The library ships the favicons and exports their tags.')
