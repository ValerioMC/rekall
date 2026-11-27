import { createPinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { skipsPermissions } from '@/common/config/claude-launch'
import { codexPreferences, launchChoices } from '@/common/config/session-launch'
import { fetchCodexModels } from '@/features/codex/api/codex.api'
import CodexSection from '@/features/settings/components/CodexSection.vue'
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

  it('reads the catalog again on the next opening and launches with the new release', async () => {
    const pinia = createPinia()
    const known = { model: 'gpt-test', displayName: 'GPT Test', supportedReasoningEfforts: [{ reasoningEffort: 'high' }] }
    const released = { model: 'gpt-example-7', displayName: 'GPT Example 7', supportedReasoningEfforts: [{ reasoningEffort: 'high' }] }
    vi.mocked(fetchCodexModels).mockResolvedValueOnce([known]).mockResolvedValueOnce([known, released])
    const first = mount(CodexSection, { global: { plugins: [pinia] } })
    await flushPromises()
    expect(first.find('option[value="gpt-example-7"]').exists()).toBe(false)
    first.unmount()

    const section = mount(CodexSection, { global: { plugins: [pinia] } })
    await flushPromises()
    await section.get('select#codex-model').setValue('gpt-example-7')
    await section.get('select#codex-effort').setValue('high')

    expect(section.get('option[value="gpt-example-7"]').text()).toBe('GPT Example 7')
    expect(launchChoices('CODEX')).toMatchObject({ model: 'gpt-example-7', effort: 'high' })
  })

  it('offers models and keeps the effort closed until a model is chosen', async () => {
    const section = mount(CodexSection, { global: { plugins: [createPinia()] } })
    await flushPromises()
    const model = section.get<HTMLSelectElement>('select#codex-model')
    const effort = section.get<HTMLSelectElement>('select#codex-effort')
    expect(effort.element.disabled).toBe(true)

    await model.setValue('gpt-test')
    await effort.setValue('high')

    expect(effort.element.disabled).toBe(false)
    expect(codexPreferences().model).toBe('gpt-test')
    expect(codexPreferences().effort).toBe('high')
  })
})
