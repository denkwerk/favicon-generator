export default defineNuxtConfig({
  modules: ['@denkwerk/nuxt-favicon-generator'],

  favicon: {
    input: './assets/favicon.svg',
    themeColor: '#02969c',
    manifest: { name: 'Favicon Generator Nuxt Example', shortName: 'Nuxt Example' },
  },

  compatibilityDate: '2026-09-01',
  devtools: { enabled: false },
})
