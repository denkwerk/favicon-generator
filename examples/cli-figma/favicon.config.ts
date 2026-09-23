import { existsSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { defineConfig } from '@denkwerk/favicon-generator'

// The token comes from the FIGMA_TOKEN env var or, locally, from the
// gitignored .env.local at the repository root (see .env.example).
const envFile = fileURLToPath(new URL('../../.env.local', import.meta.url))
if (existsSync(envFile)) {
  process.loadEnvFile(envFile)
}

export default defineConfig({
  input: 'https://www.figma.com/design/77SgSAGXbDv1Eye6htYdCG/ONE---Assets-Library-NEW?node-id=19938-42',
  figmaToken: process.env.FIGMA_TOKEN,
  output: './public/favicons',
  overwrite: true,
  pathPrefix: '/favicons/',
  appName: 'Favicon Generator Figma Example',
  appShortName: 'Figma Example',
  themeColor: '#02969c',
})
