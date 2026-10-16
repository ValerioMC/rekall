import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import SettingsTrigger from '@/components/console/SettingsTrigger.vue'
import { useClaudeConnectionStore } from '@/features/claude/store/claudeConnection.store'
import type { ClaudeConnectionStatus } from '@/features/claude/model/claude'

vi.mock('@/features/claude/api/claude.api', () => ({ fetchClaudeInstallation: vi.fn(), installClaudeIntegration: vi.fn() }))

function mountWith(status: ClaudeConnectionStatus | null) {
  const connection = useClaudeConnectionStore()
  if (status) {
    connection.installation = {
      status,
      endpoint: 'http://localhost:47355/mcp',
      registeredUrl: null,
      folderScoped: [],
      commandInstalled: true,
      cliPath: null,
      manualCommand: ''
    }
  }
  return mount(SettingsTrigger)
}

describe('SettingsTrigger', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('shows no warning while Claude Code is connected', () => {
    const wrapper = mountWith('CONNECTED')

    expect(wrapper.find('[data-testid="settings-warning"]').exists()).toBe(false)
    expect(wrapper.get('[data-testid="settings-trigger"]').attributes('aria-label')).toBe('Settings')
  })

  it('shows no warning before the registration has been read', () => {
    expect(mountWith(null).find('[data-testid="settings-warning"]').exists()).toBe(false)
  })

  it('warns, and says why, when the registration needs repair', () => {
    const wrapper = mountWith('OUTDATED')

    expect(wrapper.find('[data-testid="settings-warning"]').exists()).toBe(true)
    expect(wrapper.get('[data-testid="settings-trigger"]').attributes('aria-label')).toContain('needs repair')
  })

  it('opens settings on click', async () => {
    const wrapper = mountWith('OUTDATED')

    await wrapper.get('[data-testid="settings-trigger"]').trigger('click')

    expect(wrapper.emitted('open')).toHaveLength(1)
  })
})
