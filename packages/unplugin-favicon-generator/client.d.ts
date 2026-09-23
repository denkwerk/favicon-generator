// Types for `virtual:favicons` and its alias `~favicons` (for webpack and Rspack). Add `@denkwerk/unplugin-favicon-generator/client`
// to `compilerOptions.types`, or reference it: /// <reference types="@denkwerk/unplugin-favicon-generator/client" />
declare module 'virtual:favicons' {
  /** Attributes of one `<link>` or `<meta>` tag. */
  export type HeadTag = Record<string, string>
  /** The `<link>` tags: icons, Apple touch icons, manifest. */
  export const link: HeadTag[]
  /** The `<meta>` tags: theme color, Windows tiles. */
  export const meta: HeadTag[]
  /** All tags as HTML, one per line. */
  export const html: string
  const favicons: { link: HeadTag[], meta: HeadTag[], html: string }
  export default favicons
}

declare module '~favicons' {
  /** Attributes of one `<link>` or `<meta>` tag. */
  export type HeadTag = Record<string, string>
  /** The `<link>` tags: icons, Apple touch icons, manifest. */
  export const link: HeadTag[]
  /** The `<meta>` tags: theme color, Windows tiles. */
  export const meta: HeadTag[]
  /** All tags as HTML, one per line. */
  export const html: string
  const favicons: { link: HeadTag[], meta: HeadTag[], html: string }
  export default favicons
}
