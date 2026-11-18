import { createPinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { skipsPermissions } from '@/common/config/claude-launch'
import { codexPreferences } from '@/common/config/session-launch'
import CodexSection from '@/features/settings/components/CodexSection.vue'
import SessionChoiceGroup from '@/features/settings/components/SessionChoiceGroup.vue'
import SessionPermissionSwitch from '@/features/settings/components/SessionPermissionSwitch.vue'
import { installStorage } from '../../support/memoryStorage'

vi.mock('@/features/codex/api/codex.api', () => ({
  fetchCodexSetup: vi.fn().mockResolvedValue({ status: 'CONNECTED' }),
  fetchCodexModels: vi.fn().mockResolvedValue([
    { model: 'gpt-test', displayName: 'GPT Test', supportedReasoningEfforts: [{ reasoningEffort: 'high' }] }
  ])
}))

describe('Codex session settings', () => {
  beforeEach(installStorage)

  it('uses the shared danger switch and saves the Codex setting independently', async () => {
    const section = mount(CodexSection, { global: { plugins: [createPinia()] } })
    await flushPromises()

    const control = section.getComponent(SessionPermissionSwitch).get('[role="switch"]')
    expect(control.attributes('aria-checked')).toBe('false')
    await control.trigger('click')
    expect(control.attributes('aria-checked')).toBe('true')
    expect(control.classes()).toContain('bg-danger-soft')
    expect(codexPreferences().skipPermissions).toBe(true)
    expect(skipsPermissions()).toBe(false)

    await control.trigger('click')
    expect(control.attributes('aria-checked')).toBe('false')
    expect(codexPreferences().skipPermissions).toBe(false)
  })

  it('offers models and their supported efforts with the shared choice controls', async () => {
    const section = mount(CodexSection, { global: { plugins: [createPinia()] } })
    await flushPromises()
    expect(section.findAllComponents(SessionChoiceGroup)).toHaveLength(2)
    expect(section.get('[data-testid="codex-effort-default"]').attributes('disabled')).toBeDefined()

    const model = section.get('[data-testid="codex-model-gpt-test"]')
    await model.trigger('click')
    expect(model.attributes('aria-checked')).toBe('true')

    const effort = section.get('[data-testid="codex-effort-high"]')
    await effort.trigger('click')
    expect(effort.attributes('aria-checked')).toBe('true')
    expect(codexPreferences().model).toBe('gpt-test')
    expect(codexPreferences().effort).toBe('high')
  })
})
