import { defineConfig } from '@denkwerk/favicon-generator'

export default defineConfig({
  input: './assets/favicon.svg',
  themeColor: '#c2185b',
  manifest: {
    name: 'Favicon Generator Nuxt Config File Example',
    shortName: 'Config File Example',
    backgroundColor: '#fdf2f6',
  },
})
