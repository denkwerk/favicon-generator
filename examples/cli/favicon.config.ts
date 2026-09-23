import { existsSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { defineConfig } from '@denkwerk/favicon-generator'

const FIGMA_URL = 'https://www.figma.com/design/77SgSAGXbDv1Eye6htYdCG/ONE---Assets-Library-NEW?node-id=19938-42'

// A token in FIGMA_TOKEN or the (gitignored) .figma-token at the repository
// root exports the icon straight from Figma; otherwise the local copy is used.
const tokenFile = fileURLToPath(new URL('../../.figma-token', import.meta.url))
const hasFigmaToken = Boolean(process.env.FIGMA_TOKEN) || existsSync(tokenFile)

export default defineConfig({
  input: hasFigmaToken ? FIGMA_URL : './assets/favicon.svg',
  figmaTokenFile: existsSync(tokenFile) ? tokenFile : undefined,
  output: './public/favicons',
  overwrite: true,
  pathPrefix: '/favicons/',
  appName: 'Favicon Generator CLI Example',
  appShortName: 'CLI Example',
  themeColor: '#02969c',
  manifestCrossorigin: 'use-credentials',
})
