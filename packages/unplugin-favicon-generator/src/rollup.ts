import { createUnplugin } from 'unplugin'
import { unpluginFactory } from './index.js'

/** The favicon-generator plugin for Rollup. */
export default createUnplugin(unpluginFactory).rollup
