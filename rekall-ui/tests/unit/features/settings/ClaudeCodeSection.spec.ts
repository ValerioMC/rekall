import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import ClaudeCodeSection from '@/features/settings/components/ClaudeCodeSection.vue'
import { preferredEffort, preferredModel } from '@/common/config/claude-launch'
import { installStorage } from '../../support/memoryStorage'
import { useClaudeCommandUpdateStore } from '@/features/claude/store/claudeCommandUpdate.store'
import type { ClaudeInstallation } from '@/features/claude/model/claude'

/**
 * The Claude Code section, whose whole job is to tell the truth about a registration nobody can
 * see from inside the application, and to offer the one action that fixes it. Every state is
 * worth a test because each one is a different sentence and a different button.
 */
const fetchClaudeInstallation = vi.fn()
const installClaudeIntegration = vi.fn()

vi.mock('@/features/claude/api/claude.api', () => ({
  fetchClaudeInstallation: () => fetchClaudeInstallation(),
  installClaudeIntegration: () => installClaudeIntegration(),
  fetchCommandUpdateState: vi.fn()
}))

const ENDPOINT = 'http://localhost:47355/mcp'

function installation(overrides: Partial<ClaudeInstallation> = {}): ClaudeInstallation {
  return {
    status: 'NOT_CONNECTED',
    endpoint: ENDPOINT,
    registeredUrl: null,
    folderScoped: [],
    commandInstalled: false,
    cliPath: '/Users/someone/.local/bin/claude',
    manualCommand: `claude mcp add --scope user --transport http rekall ${ENDPOINT}`,
    ...overrides
  }
}

async function mountSection() {
  const wrapper = mount(ClaudeCodeSection)
  await flushPromises()
  return wrapper
}

describe('ClaudeCodeSection', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    installStorage()
    fetchClaudeInstallation.mockReset()
    installClaudeIntegration.mockReset()
    fetchClaudeInstallation.mockResolvedValue(installation())
  })

  it('offers to connect when Claude Code knows nothing about this instance', async () => {
    const wrapper = await mountSection()

    expect(wrapper.get('[data-testid="claude-status"]').text()).toBe('Not connected')
    expect(wrapper.get('[data-testid="claude-install"]').text()).toBe('Connect')
  })

  it('registers in one click and then says the running session will not see it', async () => {
    installClaudeIntegration.mockResolvedValue(
      installation({ status: 'CONNECTED', registeredUrl: ENDPOINT, commandInstalled: true })
    )
    const wrapper = await mountSection()

    await wrapper.get('[data-testid="claude-install"]').trigger('click')
    await flushPromises()

    expect(installClaudeIntegration).toHaveBeenCalledOnce()
    expect(wrapper.get('[data-testid="claude-status"]').text()).toBe('Connected')
    expect(wrapper.get('[data-testid="claude-install"]').text()).toBe('Reinstall')
    expect(wrapper.get('[data-testid="claude-restart-hint"]').text()).toContain('needs restarting')
  })

  it('names the address a stale registration points at, rather than only calling it wrong', async () => {
    fetchClaudeInstallation.mockResolvedValue(
      installation({ status: 'OUTDATED', registeredUrl: 'http://localhost:8080/mcp', commandInstalled: true })
    )
    const wrapper = await mountSection()

    expect(wrapper.get('[data-testid="claude-status"]').text()).toBe('Out of date')
    expect(wrapper.text()).toContain('http://localhost:8080/mcp')
    expect(wrapper.get('[data-testid="claude-install"]').text()).toBe('Repair')
  })

  it('with no CLI to run, hands over the command instead of a button that cannot work', async () => {
    fetchClaudeInstallation.mockResolvedValue(installation({ status: 'CLI_MISSING', cliPath: null }))
    const wrapper = await mountSection()

    expect(wrapper.find('[data-testid="claude-install"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="claude-copy"]').exists()).toBe(true)
    expect(wrapper.text()).toContain(`claude mcp add --scope user --transport http rekall ${ENDPOINT}`)
  })

  it('says a folder keeping its own setup is what has to be repaired, without naming scopes', async () => {
    fetchClaudeInstallation.mockResolvedValue(
      installation({
        status: 'OUTDATED',
        registeredUrl: ENDPOINT,
        commandInstalled: true,
        folderScoped: ['/Users/someone/Projects/rekall']
      })
    )
    const wrapper = await mountSection()

    expect(wrapper.text()).toContain('One folder keeps a setup of its own')
    expect(wrapper.get('[data-testid="claude-install"]').text()).toBe('Repair')
  })

  it('offers a model and a reasoning-effort level, and keeps each pick', async () => {
    fetchClaudeInstallation.mockResolvedValue(installation({ status: 'CONNECTED' }))
    const wrapper = await mountSection()
    const model = wrapper.get<HTMLSelectElement>('select#claude-model')
    const effort = wrapper.get<HTMLSelectElement>('select#claude-effort')

    expect(model.element.value).toBe('default')
    expect(model.findAll('option').map((option) => option.attributes('value'))).toContain('fable')
    expect(effort.element.value).toBe('default')
    expect(effort.findAll('option').map((option) => option.attributes('value'))).toContain('xhigh')

    await model.setValue('opus')
    await effort.setValue('high')

    expect(preferredModel()).toBe('opus')
    expect(preferredEffort()).toBe('high')
  })

  it('leaves the section usable when the configuration cannot be read', async () => {
    fetchClaudeInstallation.mockRejectedValue(new Error('nope'))
    const wrapper = await mountSection()

    expect(wrapper.text()).toContain("Couldn't read the Claude Code configuration")
    expect(wrapper.find('[data-testid="claude-install"]').exists()).toBe(false)
  })
  describe('while the /rk commands update', () => {
    async function mountOutdatedWhileUpdating(updating: boolean) {
      fetchClaudeInstallation.mockResolvedValue(installation({ status: 'OUTDATED', commandInstalled: true }))
      const wrapper = mount(ClaudeCodeSection)
      useClaudeCommandUpdateStore().commandUpdate = { state: updating ? 'UPDATING' : 'IDLE' }
      await flushPromises()
      return wrapper
    }

    it('shows the spinner and the words in place of the status and Repair', async () => {
      const wrapper = await mountOutdatedWhileUpdating(true)

      expect(wrapper.get('[data-testid="claude-status"]').text()).toBe('rk commands updating')
      expect(wrapper.find('[data-testid="step-seal"]').exists()).toBe(true)
      expect(wrapper.find('[data-testid="claude-install"]').exists()).toBe(false)
    })

    it('keeps the warning and Repair when no update runs', async () => {
      const wrapper = await mountOutdatedWhileUpdating(false)

      expect(wrapper.get('[data-testid="claude-status"]').text()).toBe('Out of date')
      expect(wrapper.find('[data-testid="step-seal"]').exists()).toBe(false)
      expect(wrapper.get('[data-testid="claude-install"]').text()).toBe('Repair')
    })
  })
})
