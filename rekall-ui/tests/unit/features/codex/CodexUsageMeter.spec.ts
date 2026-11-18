import { createPinia } from 'pinia'
import { enableAutoUnmount, flushPromises, mount } from '@vue/test-utils'
import { afterEach, describe, expect, it, vi } from 'vitest'
import AccountUsageMeter from '@/components/console/AccountUsageMeter.vue'
import HeaderStatus from '@/components/console/HeaderStatus.vue'
import { fetchClaudeUsage } from '@/features/claude/api/claude.api'
import { fetchCodexUsage } from '@/features/codex/api/codex.api'

vi.mock('@/features/codex/api/codex.api', () => ({ fetchCodexUsage: vi.fn() }))
vi.mock('@/features/claude/api/claude.api', () => ({ fetchClaudeUsage: vi.fn() }))

enableAutoUnmount(afterEach)

function mountHeader() {
  vi.mocked(fetchClaudeUsage).mockResolvedValue({
    status: 'OK',
    fetchedAt: '',
    retryAt: null,
    limits: [{ key: 'session', label: 'Session', percent: 61, severity: 'NORMAL', resetsAt: null }]
  })
  return mount(HeaderStatus, { global: { plugins: [createPinia()], stubs: { QueueBeacon: true } } })
}

describe('account usage in the header', () => {
  it('renders the same ring and hover and keyboard interactions for each provider', async () => {
    vi.mocked(fetchCodexUsage).mockResolvedValue({
      status: 'OK',
      primary: { usedPercent: 27, resetsAt: null, windowDurationMins: 300 },
      secondary: { usedPercent: 53, resetsAt: null, windowDurationMins: 10080 }
    })
    const header = mountHeader()
    await flushPromises()

    expect(header.findAllComponents(AccountUsageMeter)).toHaveLength(2)
    for (const [provider, name, percent] of [['claude', 'Claude', 61], ['codex', 'Codex', 27]] as const) {
      const meter = header.get(`[data-testid="${provider}-usage"]`)
      const trigger = meter.get('button')
      const arcs = trigger.findAll('circle')
      expect(arcs).toHaveLength(2)
      expect(Number(arcs[1]?.attributes('stroke-dashoffset'))).toBeCloseTo(2 * Math.PI * 8 * (1 - percent / 100))
      expect(trigger.attributes('aria-label')).toBe(`${name} session at ${percent}%`)

      await meter.trigger('mouseenter')
      expect(meter.get('[role="group"]').attributes('aria-label')).toBe(`${name} usage`)
      await meter.trigger('mouseleave')
      expect(meter.find('[role="group"]').exists()).toBe(false)

      await trigger.trigger('focusin')
      expect(trigger.attributes('aria-expanded')).toBe('true')
      await trigger.trigger('keydown', { key: 'Escape' })
      expect(trigger.attributes('aria-expanded')).toBe('false')
    }

    const codex = header.get('[data-testid="codex-usage"]')
    await codex.trigger('mouseenter')
    expect(codex.text()).toContain('5-hour window')
    expect(codex.text()).toContain('7-day window')
    expect(codex.text()).toContain('53%')
    expect(codex.findAll('li [style]').map((bar) => bar.attributes('style'))).toEqual(['width: 27%;', 'width: 53%;'])

    await codex.get('[data-testid="usage-check-again"]').trigger('click')
    expect(fetchCodexUsage).toHaveBeenLastCalledWith(true)
    expect(fetchClaudeUsage).toHaveBeenCalledWith(false)
  })

  it('omits the Codex meter when the CLI is missing', async () => {
    vi.mocked(fetchCodexUsage).mockResolvedValue({ status: 'CLI_MISSING' })
    const header = mountHeader()
    await flushPromises()
    expect(header.find('[data-testid="codex-usage"]').exists()).toBe(false)
    expect(header.findAllComponents(AccountUsageMeter)).toHaveLength(1)
  })
})
