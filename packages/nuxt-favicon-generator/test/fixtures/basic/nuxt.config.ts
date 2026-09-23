import FaviconModule from '../../../src/module'

export default defineNuxtConfig({
  modules: [FaviconModule],
  favicon: { input: './logo.svg' },
  compatibilityDate: '2026-09-01',
})
