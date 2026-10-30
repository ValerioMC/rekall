import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import LangfuseSection from '@/features/settings/components/LangfuseSection.vue'
import type { TelemetrySettings } from '@/features/settings/model/telemetry'

/**
 * The Langfuse section writes a secret the server keeps in the credential store. It has to send
 * that secret only when one was typed, and never show the one already stored.
 */
const fetchTelemetrySettings = vi.fn()
const saveTelemetrySettings = vi.fn()

vi.mock('@/features/settings/api/telemetry.api', () => ({
  fetchTelemetrySettings: () => fetchTelemetrySettings(),
  saveTelemetrySettings: (...args: unknown[]) => saveTelemetrySettings(...args)
}))

function settings(overrides: Partial<TelemetrySettings> = {}): TelemetrySettings {
  return {
    enabled: false,
    host: 'http://localhost:3000',
    publicKey: '',
    logContent: false,
    secretSaved: false,
    ...overrides
  }
}

async function mountSection() {
  const wrapper = mount(LangfuseSection)
  await flushPromises()
  return wrapper
}

describe('LangfuseSection', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    fetchTelemetrySettings.mockReset()
    saveTelemetrySettings.mockReset()
    fetchTelemetrySettings.mockResolvedValue(settings())
  })

  it('starts on the default host with tracing off', async () => {
    const wrapper = await mountSection()

    expect((wrapper.get('[data-testid="langfuse-host"]').element as HTMLInputElement).value).toBe(
      'http://localhost:3000'
    )
    expect(wrapper.get('[data-testid="langfuse-enabled"]').attributes('aria-checked')).toBe('false')
  })

  it('sends the secret key when one was typed', async () => {
    saveTelemetrySettings.mockResolvedValue(
      settings({ enabled: true, publicKey: 'pk-lf-1', secretSaved: true })
    )
    const wrapper = await mountSection()

    await wrapper.get('[data-testid="langfuse-enabled"]').trigger('click')
    await wrapper.get('[data-testid="langfuse-public-key"]').setValue('pk-lf-1')
    await wrapper.get('[data-testid="langfuse-secret-key"]').setValue('sk-lf-1')
    await wrapper.get('form').trigger('submit')
    await flushPromises()

    expect(saveTelemetrySettings).toHaveBeenCalledWith({
      enabled: true,
      host: 'http://localhost:3000',
      publicKey: 'pk-lf-1',
      logContent: false,
      secretKey: 'sk-lf-1'
    })
  })

  it('never shows a stored secret and leaves it out of a save that did not retype it', async () => {
    fetchTelemetrySettings.mockResolvedValue(
      settings({ enabled: true, publicKey: 'pk-lf-1', secretSaved: true })
    )
    saveTelemetrySettings.mockResolvedValue(
      settings({ enabled: true, publicKey: 'pk-lf-1', secretSaved: true })
    )
    const wrapper = await mountSection()

    expect(wrapper.find('[data-testid="langfuse-secret-key"]').exists()).toBe(false)
    expect(wrapper.get('[data-testid="langfuse-secret-saved"]').text()).toBe('set')

    await wrapper.get('form').trigger('submit')
    await flushPromises()

    expect(saveTelemetrySettings).toHaveBeenCalledOnce()
    expect(saveTelemetrySettings.mock.calls[0]?.[0]).not.toHaveProperty('secretKey')
  })

  it('offers an empty field to replace the stored secret and forgets it after saving', async () => {
    fetchTelemetrySettings.mockResolvedValue(settings({ publicKey: 'pk-lf-1', secretSaved: true }))
    saveTelemetrySettings.mockResolvedValue(settings({ publicKey: 'pk-lf-1', secretSaved: true }))
    const wrapper = await mountSection()

    await wrapper.get('[data-testid="langfuse-replace-secret"]').trigger('click')
    const field = wrapper.get('[data-testid="langfuse-secret-key"]')
    expect((field.element as HTMLInputElement).value).toBe('')
    await field.setValue('sk-lf-2')
    await wrapper.get('form').trigger('submit')
    await flushPromises()

    expect(saveTelemetrySettings.mock.calls[0]?.[0]).toMatchObject({ secretKey: 'sk-lf-2' })
    expect(wrapper.find('[data-testid="langfuse-secret-key"]').exists()).toBe(false)
    expect(wrapper.html()).not.toContain('sk-lf-2')
  })
})
