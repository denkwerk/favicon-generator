import { defineConfig } from '@denkwerk/favicon-generator'

export default defineConfig({
  input: './assets/favicon.svg',
  output: './public/favicons',
  overwrite: true,
  pathPrefix: '/favicons/',
  appName: 'Favicon Generator CLI Example',
  appShortName: 'CLI Example',
  themeColor: '#02969c',
  manifestCrossorigin: 'use-credentials',
})
