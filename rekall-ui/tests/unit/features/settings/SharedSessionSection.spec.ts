import { beforeEach, describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import SharedSessionSection from '@/features/settings/components/SharedSessionSection.vue'
import { useClaudeConnectionStore } from '@/features/claude/store/claudeConnection.store'
import { installStorage } from '../../support/memoryStorage'

const ENDPOINT = 'http://localhost:47355/mcp'

describe('Settings > Both providers', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    installStorage()
  })

  it('shows the MCP address once the installation is read', async () => {
    const section = mount(SharedSessionSection)
    const address = section.get('[data-testid="mcp-endpoint"]')
    expect(address.text()).toBe('Not read yet')

    useClaudeConnectionStore().installation = {
      status: 'CONNECTED',
      endpoint: ENDPOINT,
      registeredUrl: ENDPOINT,
      folderScoped: [],
      commandInstalled: true,
      cliPath: '/bin/claude',
      manualCommand: ''
    }
    await section.vm.$nextTick()

    expect(address.text()).toBe(ENDPOINT)
  })
})
