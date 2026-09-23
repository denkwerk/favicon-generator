import { html, link, meta } from 'virtual:favicons'

/** Attributes of one `<link>` or `<meta>` tag. */
export type HeadTag = Record<string, string>

// Typed explicitly: an inferred type would point the published .d.ts at
// `virtual:favicons`, which only exists while this library is built.

/** The `<link>` and `<meta>` tags for the favicons this library ships in `dist/favicons/`. */
export const faviconHead: { link: HeadTag[], meta: HeadTag[] } = { link, meta }

/** The same tags as HTML, e.g. for a server-rendered template. */
export const faviconHtml: string = html
