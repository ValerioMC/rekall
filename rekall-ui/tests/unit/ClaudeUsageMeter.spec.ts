import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { enableAutoUnmount, flushPromises, mount } from '@vue/test-utils'
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
  retryAt: null,
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

const SIGNED_OUT: ClaudeUsage = {
  status: 'UNAUTHENTICATED',
  limits: [],
  fetchedAt: '',
  retryAt: null
}

const RATE_LIMITED: ClaudeUsage = {
  status: 'RATE_LIMITED',
  limits: [],
  fetchedAt: '2026-09-08T12:00:00Z',
  retryAt: '2026-09-08T12:47:00Z'
}

describe('ClaudeUsageMeter', () => {
  // The meter listens on `window`; a mount left behind would still answer the next test's focus.
  enableAutoUnmount(afterEach)

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

  it('reads every five minutes on screen, blank or not, and never faster', async () => {
    fetchClaudeUsage.mockResolvedValue(SIGNED_OUT)
    mount(ClaudeUsageMeter)
    await flushPromises()
    expect(fetchClaudeUsage).toHaveBeenCalledTimes(1)

    await vi.advanceTimersByTimeAsync(4 * 60_000 + 30_000)
    expect(fetchClaudeUsage).toHaveBeenCalledTimes(1)

    await vi.advanceTimersByTimeAsync(30_000)
    expect(fetchClaudeUsage).toHaveBeenCalledTimes(2)

    fetchClaudeUsage.mockResolvedValue(OK)
    await vi.advanceTimersByTimeAsync(5 * 60_000)
    expect(fetchClaudeUsage).toHaveBeenCalledTimes(3)
    await vi.advanceTimersByTimeAsync(4 * 60_000)
    expect(fetchClaudeUsage).toHaveBeenCalledTimes(3)
  })

  it('takes a fresh reading when the window comes back after a minute away, not sooner', async () => {
    fetchClaudeUsage.mockResolvedValue(OK)
    mount(ClaudeUsageMeter)
    await flushPromises()

    await vi.advanceTimersByTimeAsync(30_000)
    window.dispatchEvent(new Event('focus'))
    await flushPromises()
    expect(fetchClaudeUsage).toHaveBeenCalledTimes(1)

    await vi.advanceTimersByTimeAsync(31_000)
    window.dispatchEvent(new Event('focus'))
    await flushPromises()
    expect(fetchClaudeUsage).toHaveBeenCalledTimes(2)
  })

  it('waits out a rate limit: says for how long, offers no check, then reads once it lapses', async () => {
    fetchClaudeUsage.mockResolvedValue(RATE_LIMITED)
    const wrapper = mount(ClaudeUsageMeter)
    await flushPromises()

    const root = wrapper.get('[data-testid="claude-usage"]')
    expect(root.attributes('data-state')).toBe('rate-limited')
    expect(root.get('button').text()).toContain('Wait 47m')

    await root.trigger('mouseenter')
    expect(wrapper.get('[role="group"]').text()).toContain('next reading in 47m')
    expect(wrapper.find('[data-testid="usage-check-again"]').exists()).toBe(false)

    await root.get('button').trigger('click')
    window.dispatchEvent(new Event('focus'))
    await vi.advanceTimersByTimeAsync(40 * 60_000)
    expect(fetchClaudeUsage).toHaveBeenCalledTimes(1)

    fetchClaudeUsage.mockResolvedValue(OK)
    await vi.advanceTimersByTimeAsync(8 * 60_000)
    expect(fetchClaudeUsage).toHaveBeenCalledTimes(2)
    expect(root.attributes('data-state')).toBe('ok')
  })

  it('keeps the last figures during a hold, says they are old, and disables checking again', async () => {
    fetchClaudeUsage.mockResolvedValue({ ...OK, retryAt: '2026-09-08T12:10:00Z' })
    const wrapper = mount(ClaudeUsageMeter)
    await flushPromises()

    expect(wrapper.get('button').text()).toContain('80%')
    await wrapper.get('[data-testid="claude-usage"]').trigger('mouseenter')
    const popover = wrapper.get('[role="group"]')
    expect(popover.get('[data-testid="usage-hold"]').text()).toContain('next reading can be taken in 10m')
    expect(popover.get('[data-testid="usage-check-again"]').attributes('disabled')).toBeDefined()

    await popover.get('[data-testid="usage-check-again"]').trigger('click')
    expect(fetchClaudeUsage).toHaveBeenCalledTimes(1)
  })
})
