// Checks the build (`vite build`): the generated files are in the output root
// and the head tags are in index.html.
import { existsSync, readFileSync } from 'node:fs'

const output = new URL('../dist/', import.meta.url)
const failures: string[] = []

for (const file of ['favicon.ico', 'favicon.svg', 'favicon-96x96.png', 'favicon-192x192.png', 'apple-touch-icon.png', 'manifest.json']) {
  if (!existsSync(new URL(file, output))) {
    failures.push(`missing ${file}`)
  }
}

const html = readFileSync(new URL('index.html', output), 'utf8')
for (const tag of [
  '<link rel="icon" type="image/svg+xml" href="/favicon.svg">',
  '<link rel="apple-touch-icon" href="/apple-touch-icon.png">',
  '<link rel="manifest" href="/manifest.json">',
  '<meta name="theme-color" content="#02969c">',
]) {
  if (!html.includes(tag)) {
    failures.push(`index.html lacks ${tag}`)
  }
}

const manifest = JSON.parse(readFileSync(new URL('manifest.json', output), 'utf8'))
if (manifest.name !== 'Favicon Generator Vite Example') {
  failures.push(`unexpected manifest name ${manifest.name}`)
}

if (failures.length > 0) {
  console.error(failures.join('\n'))
  process.exit(1)
}
console.log('Favicons and head tags are in place.')
