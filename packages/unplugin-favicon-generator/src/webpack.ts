import { createUnplugin } from 'unplugin'
import { unpluginFactory } from './index.js'

/** The favicon-generator plugin for Webpack. */
export default createUnplugin(unpluginFactory).webpack
