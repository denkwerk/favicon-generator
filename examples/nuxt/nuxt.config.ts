export default defineNuxtConfig({
  modules: ['@denkwerk/nuxt-favicon-generator'],

  faviconGenerator: {
    input: './assets/favicon.svg',
    appName: 'Favicon Generator Nuxt Example',
    appShortName: 'Nuxt Example',
    themeColor: '#02969c',
  },

  compatibilityDate: '2026-09-01',
  devtools: { enabled: false },
})
