import { mount, type DOMWrapper } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createMemoryHistory, createRouter } from 'vue-router'
import ProviderReportPage from '@/features/provider-report/components/ProviderReportPage.vue'
import type { TokenDay, TokenTally } from '@/features/tokens/model/token-usage'
import { useTokenUsageStore } from '@/features/tokens/store/tokenUsage.store'
import { asTaskId } from '@/model/branded'
import { useConsoleStore } from '@/stores/console.store'
import { taskFixture } from '../../support/taskFixture'

const taskId = asTaskId('ta')
const unseenId = asTaskId('tb')
const tokenCounts = { inputTokens: 10, cacheCreationTokens: 20, cacheReadTokens: 30, outputTokens: 40 }
const claude: TokenDay = { taskId, day: '2026-11-24', provider: 'CLAUDE', ...tokenCounts }
const codex: TokenDay = { taskId, day: '2026-11-25', provider: 'CODEX', ...tokenCounts, outputTokens: 41 }
const october: TokenDay = { ...claude, day: '2026-10-01', cacheReadTokens: 80 }
const priorYear: TokenDay = { ...codex, day: '2025-12-01' }
const rows: TokenTally[] = [
  { ...claude, stepId: null, cacheReadTokens: 110, inputTokens: 20, outputTokens: 80, cacheCreationTokens: 40 },
  { ...codex, stepId: null, cacheReadTokens: 60, inputTokens: 20, outputTokens: 82, cacheCreationTokens: 40 }
]

async function startPage() {
  const pinia = createPinia()
  setActivePinia(pinia)
  const catalog = useConsoleStore()
  catalog.tasks = [
    { ...taskFixture(taskId), code: 'VG-1', title: 'Review the launch plan', status: 'DONE', acceptedAt: '2026-11-26T00:00:00Z' },
    { ...taskFixture(unseenId), code: 'VG-2', companyName: 'Globex', title: 'Write the release note', status: 'DONE', acceptedAt: '2026-11-25T00:00:00Z' }
  ]
  const usage = useTokenUsageStore()
  usage.tallies = rows
  usage.days = [claude, codex, october, priorYear]
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/provider-report', component: ProviderReportPage },
      { path: '/task/:id/:tab', name: 'task', component: { template: '<div />' } }
    ]
  })
  await router.push('/provider-report')
  return { wrapper: mount(ProviderReportPage, { global: { plugins: [pinia, router] } }), usage, router }
}

function cellTokens(cells: readonly DOMWrapper<Element>[]): string[] {
  return cells.map((cell) => cell.attributes('data-token-count') ?? '')
}

describe('the provider report', () => {
  beforeEach(() => {
    vi.useFakeTimers({ toFake: ['Date'] })
    vi.setSystemTime(new Date('2026-11-28T12:00:00Z'))
  })

  afterEach(() => vi.useRealTimers())

  it('shows both providers, closure credit and the token breakdown for the independent overall date scope', async () => {
    const { wrapper } = await startPage()
    const overall = wrapper.get('section[aria-labelledby="provider-report-overall"]')
    expect(overall.text()).toContain('Not recorded')
    expect(overall.text()).toContain('Both')
    expect(cellTokens(overall.findAll('.report-token-total'))).toEqual(['250', '202', '452'])

    await overall.get('select[aria-label="Overall grouping"]').setValue('month')
    await overall.get('select[aria-label="Overall period (UTC)"]').setValue('2026-10-01')
    expect(cellTokens(overall.findAll('.report-token-total'))).toEqual(['150', '0', '150'])
    expect(cellTokens(overall.findAll('td[data-provider="CLAUDE"] .report-token-parts [data-token-count]'))).toEqual(['10', '20', '80', '40'])
    expect(wrapper.get('#provider-report-tasks').text()).toBe('Tasks · November 2026')

    await overall.get('select[aria-label="Overall grouping"]').setValue('year')
    await overall.get('select[aria-label="Overall period (UTC)"]').setValue('2025-01-01')
    expect(cellTokens(overall.findAll('.report-token-total'))).toEqual(['0', '101', '101'])
    wrapper.unmount()
  })

  it('switches the period grouping, opens an empty period and shows the selected task breakdown', async () => {
    const { wrapper } = await startPage()
    expect(wrapper.get('[data-testid="segment-month"]').attributes('aria-pressed')).toBe('true')
    const row = wrapper.get('[data-task-code="VG-1"]')
    expect(row.text()).toContain('Claude Code')
    expect(row.text()).toContain('Codex')
    expect(cellTokens(row.findAll('[data-token-count]'))).toEqual(['100', '101', '250', '202'])
    expect(row.get('a').attributes('href')).toBe('/task/ta/description')
    expect(row.text()).toContain('acme')
    expect(wrapper.get('[data-task-code="VG-2"]').text()).toContain('Not recorded')
    expect(wrapper.get('[data-task-code="VG-2"]').text()).toContain('Globex')

    await wrapper.get('tr[data-period-start="2026-09-01"] button').trigger('click')
    expect(wrapper.get('#provider-report-task-detail').text()).toContain('No activity in this period')
    await wrapper.get('[data-testid="segment-week"]').trigger('click')
    expect(wrapper.get('#provider-report-tasks').text()).toBe('Tasks · 23 Nov – 29 Nov 2026')
    expect(wrapper.findAll('[data-task-code]')).toHaveLength(2)
    await wrapper.get('[data-testid="segment-year"]').trigger('click')
    expect(wrapper.get('#provider-report-tasks').text()).toBe('Tasks · 2026')
    wrapper.unmount()
  })

  it('reflects token stream replacements and opens a task from the row', async () => {
    const { wrapper, usage, router } = await startPage()
    usage.apply({ taskId, days: [codex], tallies: [{ ...codex, stepId: null }] })
    await wrapper.vm.$nextTick()
    const row = wrapper.get('[data-task-code="VG-1"]')
    expect(row.text()).not.toContain('Claude Code')
    expect(cellTokens(row.findAll('[data-token-count]'))).toEqual(['101', '101'])
    await row.get('td').trigger('click')
    await vi.waitFor(() => expect(router.currentRoute.value.fullPath).toBe('/task/ta/description'))
    wrapper.unmount()
  })

  it('copies the task command from the code cell without opening the row', async () => {
    const { wrapper, router } = await startPage()
    const row = wrapper.get('[data-task-code="VG-1"]')
    await row.get('button').trigger('click')
    expect(await navigator.clipboard.readText()).toBe('/rk project:vega task:ta')
    expect(router.currentRoute.value.fullPath).toBe('/provider-report')
    wrapper.unmount()
  })
})
