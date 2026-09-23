import FaviconModule from '../../../src/module'

export default defineNuxtConfig({
  modules: [FaviconModule],
  // Merged with favicon.config.ts; these win.
  favicon: { themeColor: '#222222', manifest: { shortName: 'Inline' } },
  compatibilityDate: '2026-09-01',
})
