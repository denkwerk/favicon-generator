import { fileURLToPath } from 'node:url'

export const fixture = (name: string) => fileURLToPath(new URL(`./fixtures/${name}`, import.meta.url))

/** The `<link>` and `<meta>` tags of a page, one per line. */
export function headTags(html: string): string[] {
  return [...html.matchAll(/<(?:link|meta)\b[^>]*>/g)].map((match) => match[0])
}

/**
 * Whether `response` is a served file. Unknown paths render the app (HTML with
 * status 200, as the fixtures have no pages), so the status alone does not tell.
 */
export function isFile(response: Response): boolean {
  return response.ok && !response.headers.get('content-type')?.includes('text/html')
}
