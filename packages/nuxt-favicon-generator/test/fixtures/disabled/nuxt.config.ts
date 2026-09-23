import FaviconModule from '../../../src/module'

export default defineNuxtConfig({
  modules: [FaviconModule],
  favicon: { input: './logo.svg', enabled: false },
  compatibilityDate: '2026-09-01',
})
