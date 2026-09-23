import { type FaviconConfig, mergeConfig } from '@denkwerk/favicon-generator'
import type { GeneratorOptions } from './module'

/**
 * Inline module options take precedence over the config file, like CLI flags
 * do; the options of a group are merged key by key. The module decides where
 * the files go, so the file's output settings are dropped.
 */
export function mergeOptions(fromFile: FaviconConfig, inline: GeneratorOptions): GeneratorOptions {
  const { output: _output, overwrite: _overwrite, snippets: _snippets, $schema: _schema, ...merged } = fromFile
  return mergeConfig<GeneratorOptions>(merged, inline)
}
