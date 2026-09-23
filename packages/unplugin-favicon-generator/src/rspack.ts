import { createUnplugin } from 'unplugin'
import { unpluginFactory } from './index.js'

/** The favicon-generator plugin for Rspack. */
export default createUnplugin(unpluginFactory).rspack
