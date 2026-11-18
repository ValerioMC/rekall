import { useClaudeConnectionStore } from '@/features/claude/store/claudeConnection.store'
import { useCodexStore } from '@/features/codex/store/codex.store'

export function installBothSessionProviders(): void {
  useClaudeConnectionStore().installation = {
    status: 'NOT_CONNECTED',
    endpoint: 'http://localhost:47355/mcp',
    registeredUrl: null,
    folderScoped: [],
    commandInstalled: false,
    cliPath: '/usr/local/bin/claude',
    manualCommand: ''
  }
  useCodexStore().setup = { status: 'NOT_CONNECTED' }
}
