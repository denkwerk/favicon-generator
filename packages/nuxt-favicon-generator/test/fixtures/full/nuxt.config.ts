import FaviconModule from '../../../src/module'

export default defineNuxtConfig({
  modules: [FaviconModule],
  app: { baseURL: '/app/' },
  favicon: {
    input: './logo.svg',
    pathPrefix: '/icons/',
    themeColor: '#02969c',
    manifest: { name: 'Full Fixture', shortName: 'Full', display: 'standalone', maskable: true },
    windows: { tileColor: '#02969c' },
  },
  compatibilityDate: '2026-09-01',
})
