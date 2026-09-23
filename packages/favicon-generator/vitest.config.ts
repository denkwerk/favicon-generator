import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    include: ['test/**/*.test.ts'],
    // Each test runs the native binary, which can take a moment on CI.
    testTimeout: 30_000,
  },
})
