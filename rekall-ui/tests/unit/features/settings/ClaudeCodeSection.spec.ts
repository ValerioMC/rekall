import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import ClaudeCodeSection from '@/features/settings/components/ClaudeCodeSection.vue'
import { preferredEffort, preferredModel, setPreferredEffort, setPreferredModel } from '@/common/config/claude-launch'
import { launchChoices } from '@/common/config/session-launch'
import { installStorage } from '../../support/memoryStorage'
import { useClaudeCommandUpdateStore } from '@/features/claude/store/claudeCommandUpdate.store'
import type { ClaudeInstallation, ClaudeModel } from '@/features/claude/model/claude'

/**
 * The Claude Code section, whose whole job is to tell the truth about a registration nobody can
 * see from inside the application, and to offer the one action that fixes it. Every state is
 * worth a test because each one is a different sentence and a different button.
 */
const fetchClaudeInstallation = vi.fn()
const installClaudeIntegration = vi.fn()
const fetchClaudeModels = vi.fn()

vi.mock('@/features/claude/api/claude.api', () => ({
  fetchClaudeInstallation: () => fetchClaudeInstallation(),
  installClaudeIntegration: () => installClaudeIntegration(),
  fetchClaudeModels: () => fetchClaudeModels(),
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

function offered(value: string, displayName: string, overrides: Partial<ClaudeModel> = {}): ClaudeModel {
  return {
    value,
    displayName,
    resolvedModel: value,
    supportsEffort: true,
    supportedEffortLevels: ['low', 'medium', 'high'],
    ...overrides
  }
}

const SONNET = offered('sonnet', 'Sonnet 5', { resolvedModel: 'claude-sonnet-5' })
const EXAMPLE = offered('claude-example-7[1m]', 'Example 7', { supportedEffortLevels: ['high'] })
const NO_EFFORT = offered('claude-quick-1', 'Quick 1', { supportsEffort: false, supportedEffortLevels: null })

function optionValues(select: { findAll: (selector: string) => { attributes: (name: string) => string | undefined }[] }) {
  return select.findAll('option').map((option) => option.attributes('value'))
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
    fetchClaudeModels.mockReset()
    fetchClaudeInstallation.mockResolvedValue(installation())
    fetchClaudeModels.mockResolvedValue([])
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
    fetchClaudeModels.mockResolvedValue([
      offered('opus', 'Opus 5', { resolvedModel: 'claude-opus-5', supportedEffortLevels: ['high', 'xhigh'] }),
      offered('fable', 'Fable 5', { resolvedModel: 'claude-fable-5' })
    ])
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
  describe('the model catalog', () => {
    it('shows a new release on the next opening, with its efforts, and carries the pick to a launch', async () => {
      fetchClaudeModels.mockResolvedValueOnce([SONNET]).mockResolvedValueOnce([SONNET, EXAMPLE])
      const first = await mountSection()
      expect(optionValues(first.get('select#claude-model'))).toEqual(['default', 'sonnet'])
      first.unmount()

      const wrapper = await mountSection()
      const model = wrapper.get<HTMLSelectElement>('select#claude-model')
      const effort = wrapper.get<HTMLSelectElement>('select#claude-effort')
      expect(optionValues(model)).toEqual(['default', 'sonnet', 'claude-example-7[1m]'])

      await model.setValue('claude-example-7[1m]')
      expect(optionValues(effort)).toEqual(['default', 'high'])
      await effort.setValue('high')

      expect(model.findAll('option')[2]!.text()).toBe('Example 7')
      expect(launchChoices('CLAUDE')).toMatchObject({ model: 'claude-example-7[1m]', effort: 'high' })
    })

    it('reads the catalog even while the connection cannot be read', async () => {
      fetchClaudeInstallation.mockRejectedValue(new Error('nope'))

      await mountSection()

      expect(fetchClaudeModels).toHaveBeenCalledOnce()
    })

    it('keeps a saved choice and the family names when the first read fails, then retries', async () => {
      setPreferredModel('claude-example-7[1m]')
      setPreferredEffort('high')
      fetchClaudeModels.mockRejectedValueOnce(new Error('silent CLI')).mockResolvedValueOnce([SONNET, EXAMPLE])
      const wrapper = await mountSection()
      const model = wrapper.get<HTMLSelectElement>('select#claude-model')

      expect(wrapper.get('[data-testid="claude-models-failed"]').text()).toContain('These are the family names')
      expect(optionValues(model)).toEqual(['default', 'sonnet', 'fable', 'opus', 'haiku', 'claude-example-7[1m]'])
      expect(model.element.value).toBe('claude-example-7[1m]')
      expect(wrapper.get<HTMLSelectElement>('select#claude-effort').element.value).toBe('high')

      await wrapper.get('[data-testid="claude-models-retry"]').trigger('click')
      await flushPromises()

      expect(wrapper.find('[data-testid="claude-models-failed"]').exists()).toBe(false)
      expect(model.element.value).toBe('claude-example-7[1m]')
      expect(preferredModel()).toBe('claude-example-7[1m]')
      expect(preferredEffort()).toBe('high')
    })

    it('keeps the last list when a later opening cannot read it', async () => {
      fetchClaudeModels.mockResolvedValueOnce([SONNET, EXAMPLE]).mockRejectedValueOnce(new Error('silent CLI'))
      const first = await mountSection()
      first.unmount()

      const wrapper = await mountSection()

      expect(wrapper.get('[data-testid="claude-models-failed"]').text()).toContain('the last list')
      expect(optionValues(wrapper.get('select#claude-model'))).toEqual(['default', 'sonnet', 'claude-example-7[1m]'])
    })

    it('never rewrites a saved choice the refreshed catalog no longer offers', async () => {
      setPreferredModel('claude-quick-1')
      setPreferredEffort('max')
      fetchClaudeModels.mockResolvedValue([NO_EFFORT])
      const wrapper = await mountSection()

      expect(wrapper.get<HTMLSelectElement>('select#claude-effort').element.value).toBe('max')
      expect(wrapper.text()).toContain('Quick 1 doesn’t list Max')
      expect(preferredEffort()).toBe('max')
    })

    it('resets an effort the picked model cannot take, and closes the list for one that takes none', async () => {
      setPreferredEffort('max')
      fetchClaudeModels.mockResolvedValue([NO_EFFORT])
      const wrapper = await mountSection()

      await wrapper.get('select#claude-model').setValue('claude-quick-1')

      expect(preferredEffort()).toBe('default')
      expect(wrapper.get<HTMLSelectElement>('select#claude-effort').element.disabled).toBe(true)
      expect(wrapper.text()).toContain('Quick 1 takes no reasoning effort')
    })
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
