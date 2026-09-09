import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import type { ClaudeUsage } from '@/model/claude'

const fetchClaudeUsage = vi.fn<() => Promise<ClaudeUsage>>()

vi.mock('@/api/claude.api', () => ({ fetchClaudeUsage: () => fetchClaudeUsage() }))

import ClaudeUsageMeter from '@/components/console/ClaudeUsageMeter.vue'

const OK: ClaudeUsage = {
  status: 'OK',
  fetchedAt: '2026-09-08T12:00:00Z',
  limits: [
    {
      key: 'session',
      label: 'Session',
      percent: 80,
      severity: 'WARNING',
      resetsAt: '2026-09-08T15:24:00Z'
    },
    {
      key: 'weekly_all',
      label: 'Weekly · all models',
      percent: 50,
      severity: 'NORMAL',
      resetsAt: '2026-09-10T18:00:00Z'
    }
  ]
}

describe('ClaudeUsageMeter', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    vi.setSystemTime(Date.parse('2026-09-08T12:00:00Z'))
    fetchClaudeUsage.mockReset()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('shows the session percentage and its countdown once usage loads', async () => {
    fetchClaudeUsage.mockResolvedValue(OK)
    const wrapper = mount(ClaudeUsageMeter)
    await flushPromises()

    const trigger = wrapper.get('[data-testid="claude-usage"] button')
    expect(trigger.text()).toContain('80%')
    expect(trigger.text()).toContain('3h 24m')
    expect(trigger.attributes('aria-label')).toBe('Claude session at 80%, resets in 3h 24m')
  })

  it('opens a row per limit on hover, each with its own reset', async () => {
    fetchClaudeUsage.mockResolvedValue(OK)
    const wrapper = mount(ClaudeUsageMeter)
    await flushPromises()

    await wrapper.get('[data-testid="claude-usage"]').trigger('mouseenter')

    const text = wrapper.get('[role="group"]').text()
    expect(text).toContain('Session')
    expect(text).toContain('Weekly · all models')
    expect(text).toContain('resets in 3h 24m')
    expect(text).toContain('resets in 2d 6h')
  })

  it('asks the user to sign in when Claude Code has no token', async () => {
    fetchClaudeUsage.mockResolvedValue({ status: 'UNAUTHENTICATED', limits: [], fetchedAt: '' })
    const wrapper = mount(ClaudeUsageMeter)
    await flushPromises()

    expect(wrapper.text()).toContain('Sign in')
  })

  it('degrades to unavailable when the request fails, without throwing', async () => {
    fetchClaudeUsage.mockRejectedValue(new Error('offline'))
    const wrapper = mount(ClaudeUsageMeter)
    await flushPromises()

    expect(wrapper.text()).toContain('Usage')
    await wrapper.get('[data-testid="claude-usage"]').trigger('mouseenter')
    expect(wrapper.get('[role="group"]').text()).toContain('could not be reached')
  })
})
