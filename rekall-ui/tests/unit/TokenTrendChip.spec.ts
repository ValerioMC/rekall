import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import TaskTokenChip from '@/features/tokens/components/TaskTokenChip.vue'
import ProjectTokenChip from '@/features/tokens/components/ProjectTokenChip.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useTokenUsageStore } from '@/features/tokens/store/tokenUsage.store'
import type { ProjectId, TaskId } from '@/model/branded'
import type { Task } from '@/features/catalog/model/catalog'
import type { TokenDay, TokenTally } from '@/features/tokens/model/token-usage'

const TASK = { id: 't1' as TaskId, projectId: 'p1' as ProjectId } as unknown as Task

const tally = (outputTokens: number): TokenTally => ({
  taskId: TASK.id,
  stepId: null,
  inputTokens: 40,
  outputTokens,
  cacheCreationTokens: 2_000,
  cacheReadTokens: 90_000
})

const day = (on: string, outputTokens: number): TokenDay => ({
  taskId: TASK.id,
  day: on,
  inputTokens: 0,
  outputTokens,
  cacheCreationTokens: 0,
  cacheReadTokens: 0
})

describe('the token chip', () => {
  beforeEach(() => {
    vi.useFakeTimers({ now: new Date('2026-10-13T12:00:00Z') })
    setActivePinia(createPinia())
  })

  afterEach(() => {
    document.body.innerHTML = ''
    vi.useRealTimers()
  })

  function mountChip(tokens: number, days: TokenDay[]) {
    const store = useTokenUsageStore()
    store.tallies = tokens > 0 ? [tally(tokens)] : []
    store.days = days
    return mount(TaskTokenChip, { props: { task: TASK }, attachTo: document.body })
  }

  it('shows what the task generated, with a sparkline once there is a trend', () => {
    const wrapper = mountChip(12_400, [day('2026-10-10', 4_000), day('2026-10-12', 8_400)])

    expect(wrapper.get('[data-testid="token-chip"]').text()).toContain('12.4k')
    expect(wrapper.find('[data-testid="token-chip-spark"]').exists()).toBe(true)
    wrapper.unmount()
  })

  it('draws no sparkline for a task that generated nothing', () => {
    const wrapper = mountChip(0, [])

    expect(wrapper.get('[data-testid="token-chip"]').text()).toContain('0')
    expect(wrapper.find('[data-testid="token-chip-spark"]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('opens the daily trend with its input and cache readings, and closes on Escape', async () => {
    const wrapper = mountChip(12_400, [day('2026-10-10', 4_000), day('2026-10-12', 8_400)])

    await wrapper.get('[data-testid="token-chip"]').trigger('click')
    const panel = document.body.querySelector('[data-testid="token-chip-panel"]')

    expect(wrapper.get('[data-testid="token-chip"]').attributes('aria-expanded')).toBe('true')
    expect(panel?.querySelector('[data-testid="token-trend-readout"]')?.textContent).toContain('12.4k')
    expect(panel?.textContent).toContain('Cache read')
    expect(panel?.textContent).toContain('90k')

    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    await flushPromises()
    await vi.runAllTimersAsync()

    expect(wrapper.get('[data-testid="token-chip"]').attributes('aria-expanded')).toBe('false')
    expect(document.body.querySelector('[data-testid="token-chip-panel"]')).toBeNull()
    wrapper.unmount()
  })

  it('closes when the pointer goes down outside it', async () => {
    const wrapper = mountChip(5, [day('2026-10-12', 5)])

    await wrapper.get('[data-testid="token-chip"]').trigger('click')
    expect(document.body.querySelector('[data-testid="token-chip-panel"]')).not.toBeNull()

    document.body.dispatchEvent(new Event('pointerdown', { bubbles: true }))
    await flushPromises()
    await vi.runAllTimersAsync()

    expect(document.body.querySelector('[data-testid="token-chip-panel"]')).toBeNull()
    wrapper.unmount()
  })

  it('follows the range chosen: a month leaves out what a longer one keeps', async () => {
    const wrapper = mountChip(2_000, [day('2026-06-01', 1_500), day('2026-10-12', 500)])
    await wrapper.get('[data-testid="token-chip"]').trigger('click')
    const first = (): string => document.body.querySelector('[data-testid="token-trend-chart"] svg text:last-of-type')?.textContent ?? ''
    const readout = (): string => document.body.querySelector('[data-testid="token-trend-readout"]')?.textContent ?? ''

    expect(readout()).toContain('2k')

    ;(document.body.querySelector('[data-testid="token-trend-range-all"]') as HTMLElement).click()
    await flushPromises()

    expect(readout()).toContain('2k')
    expect(first()).toBeTruthy()
    wrapper.unmount()
  })

  it('says so when no session has generated anything on the task', async () => {
    const wrapper = mountChip(0, [])

    await wrapper.get('[data-testid="token-chip"]').trigger('click')

    expect(document.body.querySelector('[data-testid="token-trend-empty"]')).not.toBeNull()
    wrapper.unmount()
  })

  it('sums every task of a project, and leaves other projects out', async () => {
    const OTHER = { id: 't2' as TaskId, projectId: 'p1' as ProjectId } as unknown as Task
    const ELSEWHERE = { id: 't3' as TaskId, projectId: 'p2' as ProjectId } as unknown as Task
    useConsoleStore().tasks = [TASK, OTHER, ELSEWHERE]
    const store = useTokenUsageStore()
    store.tallies = [tally(1_000), { ...tally(2_000), taskId: OTHER.id }, { ...tally(9_000), taskId: ELSEWHERE.id }]
    store.days = [day('2026-10-11', 1_000), { ...day('2026-10-12', 2_000), taskId: OTHER.id }, { ...day('2026-10-12', 9_000), taskId: ELSEWHERE.id }]
    const wrapper = mount(ProjectTokenChip, { props: { projectId: 'p1' as ProjectId }, attachTo: document.body })

    await wrapper.get('[data-testid="token-chip"]').trigger('click')

    expect(wrapper.get('[data-testid="token-chip"]').text()).toContain('3k')
    expect(document.body.querySelector('[data-testid="token-trend-readout"]')?.textContent).toContain('3k')
    wrapper.unmount()
  })
})
