import { existsSync } from 'node:fs'
import { fileURLToPath } from 'node:url'

const FIGMA_URL = 'https://www.figma.com/design/77SgSAGXbDv1Eye6htYdCG/ONE---Assets-Library-NEW?node-id=19938-42'

// A token in FIGMA_TOKEN or the (gitignored) .figma-token at the repository
// root exports the icon straight from Figma; otherwise the local copy is used.
const tokenFile = fileURLToPath(new URL('../../.figma-token', import.meta.url))
const hasFigmaToken = Boolean(process.env.FIGMA_TOKEN) || existsSync(tokenFile)

export default defineNuxtConfig({
  modules: ['@denkwerk/nuxt-favicon-generator'],

  faviconGenerator: {
    input: hasFigmaToken ? FIGMA_URL : './assets/favicon.svg',
    figmaTokenFile: existsSync(tokenFile) ? tokenFile : undefined,
    appName: 'Favicon Generator Nuxt Example',
    appShortName: 'Nuxt Example',
    themeColor: '#02969c',
  },

  compatibilityDate: '2026-09-01',
  devtools: { enabled: false },
})
