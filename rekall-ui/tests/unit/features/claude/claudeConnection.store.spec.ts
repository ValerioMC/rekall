import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useClaudeConnectionStore } from '@/features/claude/store/claudeConnection.store'
import type { ClaudeConnectionStatus, ClaudeInstallation } from '@/features/claude/model/claude'

const fetchClaudeInstallation = vi.fn()
const installClaudeIntegration = vi.fn()

vi.mock('@/features/claude/api/claude.api', () => ({
  fetchClaudeInstallation: () => fetchClaudeInstallation(),
  installClaudeIntegration: () => installClaudeIntegration()
}))

function installation(status: ClaudeConnectionStatus): ClaudeInstallation {
  return {
    status,
    endpoint: 'http://localhost:47355/mcp',
    registeredUrl: null,
    folderScoped: [],
    commandInstalled: true,
    cliPath: null,
    manualCommand: ''
  }
}

describe('claude connection store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    fetchClaudeInstallation.mockReset()
    installClaudeIntegration.mockReset()
  })

  it('needs no repair before anything is read', () => {
    expect(useClaudeConnectionStore().needsRepair).toBe(false)
  })

  it('needs repair when the registration is out of date', async () => {
    fetchClaudeInstallation.mockResolvedValue(installation('OUTDATED'))
    const connection = useClaudeConnectionStore()

    await connection.load()

    expect(connection.needsRepair).toBe(true)
  })

  it.each<ClaudeConnectionStatus>(['CONNECTED', 'NOT_CONNECTED', 'CLI_MISSING'])(
    'needs no repair when the status is %s',
    async (status) => {
      fetchClaudeInstallation.mockResolvedValue(installation(status))
      const connection = useClaudeConnectionStore()

      await connection.load()

      expect(connection.needsRepair).toBe(false)
    }
  )

  it('stops needing repair once the install lands', async () => {
    fetchClaudeInstallation.mockResolvedValue(installation('OUTDATED'))
    installClaudeIntegration.mockResolvedValue(installation('CONNECTED'))
    const connection = useClaudeConnectionStore()
    await connection.load()

    await connection.install()

    expect(connection.needsRepair).toBe(false)
  })
})
