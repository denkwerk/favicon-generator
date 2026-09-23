import type { FaviconConfig } from '@denkwerk/favicon-generator'
import type { GeneratorOptions } from './module'

/** Options that are groups: `true`, `false` or an options object. */
const GROUPS = new Set(['appleTouchIcon', 'manifest', 'windows'])

const isObject = (value: unknown): value is Record<string, unknown> =>
  typeof value === 'object' && value !== null && !Array.isArray(value)

/**
 * Inline module options take precedence over the config file, like CLI flags
 * do; the options of a group are merged key by key. The module decides where
 * the files go, so the file's output settings are dropped.
 */
export function mergeOptions(fromFile: FaviconConfig, inline: GeneratorOptions): GeneratorOptions {
  const { output: _output, overwrite: _overwrite, snippets: _snippets, $schema: _schema, ...merged } = fromFile
  const result: Record<string, unknown> = merged
  for (const [key, value] of Object.entries(inline)) {
    if (value === undefined) {
      continue
    }
    const base = result[key]
    result[key] = GROUPS.has(key) && isObject(base) && isObject(value) ? { ...base, ...value } : value
  }
  return result as GeneratorOptions
}
