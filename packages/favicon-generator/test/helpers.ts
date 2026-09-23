import { mkdtempSync, readdirSync, realpathSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

export const LOGO = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><rect width="24" height="24" fill="#02969c"/></svg>'

/** A temporary project with a `package.json` (so config lookup stops there) and `logo.svg`. */
export function createProject(files: Record<string, string> = {}): string {
  // Resolved, because the binary reports real paths (e.g. /private/var on macOS).
  const dir = realpathSync(mkdtempSync(join(tmpdir(), 'favicon-generator-test-')))
  writeFileSync(join(dir, 'package.json'), '{}')
  writeFileSync(join(dir, 'logo.svg'), LOGO)
  for (const [name, contents] of Object.entries(files)) {
    writeFileSync(join(dir, name), contents)
  }
  return dir
}

export function list(dir: string): string[] {
  return readdirSync(dir).sort()
}
