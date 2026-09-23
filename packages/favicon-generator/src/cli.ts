import { spawnSync } from 'node:child_process'
import { binaryEnv, resolveBinary } from './binary.js'

let binary: string
try {
  binary = resolveBinary()
} catch (error) {
  console.error((error as Error).message)
  process.exit(1)
}

const result = spawnSync(binary, process.argv.slice(2), { stdio: 'inherit', env: binaryEnv() })
if (result.error) {
  console.error(`Failed to run ${binary}: ${result.error.message}`)
  process.exit(1)
}
if (result.signal) {
  process.kill(process.pid, result.signal)
}
process.exit(result.status ?? 1)
