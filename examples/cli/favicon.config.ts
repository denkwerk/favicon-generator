import { defineConfig } from '@denkwerk/favicon-generator'

export default defineConfig({
  input: './assets/favicon.svg',
  output: './public/favicons',
  overwrite: true,
  pathPrefix: '/favicons/',
  themeColor: '#02969c',
  manifest: {
    name: 'Favicon Generator CLI Example',
    shortName: 'CLI Example',
    crossorigin: 'use-credentials',
  },
})
