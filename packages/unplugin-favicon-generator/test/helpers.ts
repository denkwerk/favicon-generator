import { cpSync, mkdtempSync, readdirSync, realpathSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join, relative, sep } from 'node:path'
import { fileURLToPath } from 'node:url'

/**
 * A copy of `test/fixtures/<name>` in a temporary directory, so that test files
 * running in parallel do not share the cache in `node_modules/.cache`.
 */
export function copyFixture(name: string): string {
  // Real path: on macOS, tmpdir() is below the /var symlink, which bundlers resolve.
  const dir = realpathSync(mkdtempSync(join(tmpdir(), `unplugin-favicon-generator-${name}-`)))
  cpSync(fileURLToPath(new URL(`./fixtures/${name}`, import.meta.url)), dir, { recursive: true })
  return dir
}

/** The `<link>` and `<meta>` tags of a page, one per line. */
export function headTags(html: string): string[] {
  return [...html.matchAll(/<(?:link|meta)\b[^>]*>/g)].map((match) => match[0])
}

/** Files below `dir`, relative with `/` separators (also on Windows) and sorted, without directories. */
export function listFiles(dir: string): string[] {
  return readdirSync(dir, { recursive: true, withFileTypes: true })
    .filter((entry) => entry.isFile())
    .map((entry) => relative(dir, join(entry.parentPath, entry.name)).split(sep).join('/'))
    .sort()
}

/** What the default options generate. */
export const DEFAULT_FILES = ['apple-touch-icon.png', 'favicon-96x96.png', 'favicon.ico', 'favicon.svg']
