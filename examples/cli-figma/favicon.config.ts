import { existsSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { defineConfig } from '@denkwerk/favicon-generator'

// The token comes from the FIGMA_TOKEN env var or, locally, from the
// gitignored .figma-token at the repository root.
const tokenFile = fileURLToPath(new URL('../../.figma-token', import.meta.url))

export default defineConfig({
  input: 'https://www.figma.com/design/77SgSAGXbDv1Eye6htYdCG/ONE---Assets-Library-NEW?node-id=19938-42',
  figmaTokenFile: existsSync(tokenFile) ? tokenFile : undefined,
  output: './public/favicons',
  overwrite: true,
  pathPrefix: '/favicons/',
  appName: 'Favicon Generator Figma Example',
  appShortName: 'Figma Example',
  themeColor: '#02969c',
})
