import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    include: ['test/**/*.test.ts'],
    // Each test runs a real build with the native binary.
    testTimeout: 60_000,
  },
})
