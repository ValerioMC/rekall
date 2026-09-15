import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import type { ClaudeUsage } from '@/model/claude'

const fetchClaudeUsage = vi.fn<(refresh?: boolean) => Promise<ClaudeUsage>>()

vi.mock('@/api/claude.api', () => ({
  fetchClaudeUsage: (refresh?: boolean) => fetchClaudeUsage(refresh)
}))

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

const SIGNED_OUT: ClaudeUsage = { status: 'UNAUTHENTICATED', limits: [], fetchedAt: '' }

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
    expect(fetchClaudeUsage).toHaveBeenCalledWith(false)
  })

  it('sweeps the ring while the first reading is on its way', async () => {
    fetchClaudeUsage.mockReturnValue(new Promise(() => undefined))
    const wrapper = mount(ClaudeUsageMeter)
    await nextTick()

    expect(wrapper.get('[data-testid="claude-usage"]').attributes('data-state')).toBe('reading')
    expect(wrapper.find('[data-testid="usage-sweep"]').exists()).toBe(true)
    expect(wrapper.get('button').text()).toContain('Reading usage')
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
    fetchClaudeUsage.mockResolvedValue(SIGNED_OUT)
    const wrapper = mount(ClaudeUsageMeter)
    await flushPromises()

    expect(wrapper.text()).toContain('Sign in')
    expect(wrapper.get('[data-testid="claude-usage"]').attributes('data-state')).toBe('signed-out')
  })

  it('degrades to unavailable when the request fails, without throwing', async () => {
    fetchClaudeUsage.mockRejectedValue(new Error('offline'))
    const wrapper = mount(ClaudeUsageMeter)
    await flushPromises()

    expect(wrapper.text()).toContain('No reading')
    await wrapper.get('[data-testid="claude-usage"]').trigger('mouseenter')
    expect(wrapper.get('[role="group"]').text()).toContain('could not be reached')
  })

  it('checks again, past the cache, when the blank trigger is clicked', async () => {
    let deliver: (usage: ClaudeUsage) => void = () => undefined
    fetchClaudeUsage
      .mockResolvedValueOnce(SIGNED_OUT)
      .mockReturnValueOnce(new Promise<ClaudeUsage>((resolve) => (deliver = resolve)))
    const wrapper = mount(ClaudeUsageMeter)
    await flushPromises()

    await wrapper.get('[data-testid="claude-usage"] button').trigger('click')
    expect(wrapper.find('[data-testid="usage-sweep"]').exists()).toBe(true)
    expect(wrapper.get('button').text()).toContain('Sign in')
    deliver(OK)
    await flushPromises()

    expect(fetchClaudeUsage).toHaveBeenLastCalledWith(true)
    expect(wrapper.get('[data-testid="claude-usage"]').attributes('data-state')).toBe('ok')
    expect(wrapper.get('button').text()).toContain('80%')
  })

  it('offers a check-again button in the blank popover, with when it last looked', async () => {
    fetchClaudeUsage.mockResolvedValue(SIGNED_OUT)
    const wrapper = mount(ClaudeUsageMeter)
    await flushPromises()

    await wrapper.get('[data-testid="claude-usage"]').trigger('mouseenter')
    const popover = wrapper.get('[role="group"]')
    expect(popover.text()).toContain('Checked just now')

    await popover.get('[data-testid="usage-check-again"]').trigger('click')
    await flushPromises()

    expect(fetchClaudeUsage).toHaveBeenCalledTimes(2)
    expect(fetchClaudeUsage).toHaveBeenLastCalledWith(true)
  })

  it('keeps the popover toggle on click once there is a figure to show', async () => {
    fetchClaudeUsage.mockResolvedValue(OK)
    const wrapper = mount(ClaudeUsageMeter)
    await flushPromises()

    await wrapper.get('[data-testid="claude-usage"] button').trigger('click')
    expect(wrapper.find('[role="group"]').exists()).toBe(true)
    expect(fetchClaudeUsage).toHaveBeenCalledTimes(1)

    await wrapper.get('[data-testid="usage-check-again"]').trigger('click')
    await flushPromises()
    expect(fetchClaudeUsage).toHaveBeenLastCalledWith(true)
  })

  it('retries a blank reading every 15 seconds and a good one every minute', async () => {
    fetchClaudeUsage.mockResolvedValue(SIGNED_OUT)
    mount(ClaudeUsageMeter)
    await flushPromises()
    expect(fetchClaudeUsage).toHaveBeenCalledTimes(1)

    await vi.advanceTimersByTimeAsync(15_000)
    expect(fetchClaudeUsage).toHaveBeenCalledTimes(2)

    fetchClaudeUsage.mockResolvedValue(OK)
    await vi.advanceTimersByTimeAsync(15_000)
    expect(fetchClaudeUsage).toHaveBeenCalledTimes(3)

    await vi.advanceTimersByTimeAsync(45_000)
    expect(fetchClaudeUsage).toHaveBeenCalledTimes(3)
    await vi.advanceTimersByTimeAsync(15_000)
    expect(fetchClaudeUsage).toHaveBeenCalledTimes(4)
  })
})
