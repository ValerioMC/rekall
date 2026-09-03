import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import type { DOMWrapper, VueWrapper } from '@vue/test-utils'
import ReportPage from '@/components/report/ReportPage.vue'
import { router } from '@/router'
import { useConsoleStore } from '@/stores/console.store'
import type { Company, Task, TimeEntry } from '@/model/catalog'
import type { CompanyId, ProjectId, TaskId, TimeEntryId } from '@/model/branded'

/**
 * The report as a screen: the frame it opens on, what a company filter does to it, and what it
 * says when a week is empty. The arithmetic is `time-report.spec`; this is the reading of it.
 */
vi.mock('@/api/catalog.api', () => ({
  fetchCompanies: vi.fn(async () => []),
  fetchProjects: vi.fn(async () => []),
  fetchTasks: vi.fn(async () => []),
  createCompany: vi.fn(),
  updateCompany: vi.fn(),
  deleteCompany: vi.fn(),
  createProject: vi.fn(),
  updateProject: vi.fn(),
  deleteProject: vi.fn(),
  createTask: vi.fn(),
  updateTask: vi.fn(),
  deleteTask: vi.fn()
}))
vi.mock('@/api/documents.api', () => ({ fetchAllDocuments: vi.fn(async () => []) }))
vi.mock('@/api/wrapups.api', () => ({ fetchWrapups: vi.fn(async () => []) }))
vi.mock('@/api/steps.api', () => ({ fetchSteps: vi.fn(async () => []) }))
vi.mock('@/api/time-entries.api', () => ({ fetchTimeEntries: vi.fn(async () => []) }))

const acme = 'c1' as CompanyId
const globex = 'c2' as CompanyId
const vega = 'p1' as ProjectId
const beacon = 'p2' as ProjectId
const builder = 't1' as TaskId
const signal = 't2' as TaskId

// A Wednesday, with the week it belongs to running from Monday 31 August.
const TODAY = new Date(2026, 8, 2, 12, 0)

const companies: Company[] = [
  { id: acme, name: 'acme', description: null, projectCount: 1, taskCount: 1, updatedAt: '' },
  { id: globex, name: 'globex', description: null, projectCount: 1, taskCount: 1, updatedAt: '' }
]

const tasks: Task[] = [
  {
    id: builder, label: 'report-builder', title: 'Report builder', status: 'IN_PROGRESS',
    description: null, projectId: vega, projectLabel: 'vega', projectTitle: 'Vega Platform',
    companyName: 'acme', projectRepoFolder: null, documentCount: 0, stepCount: 0, stepsDone: 0, hasWrapup: false,
    anchor: 'project:vega task:report-builder', updatedAt: ''
  },
  {
    id: signal, label: 'signal-ingest', title: 'Signal ingest', status: 'TODO',
    description: null, projectId: beacon, projectLabel: 'beacon', projectTitle: 'Beacon',
    companyName: 'globex', projectRepoFolder: null, documentCount: 0, stepCount: 0, stepsDone: 0, hasWrapup: false,
    anchor: 'project:beacon task:signal-ingest', updatedAt: ''
  }
]

function session(id: string, taskId: TaskId, day: number, hours: number): TimeEntry {
  const started = new Date(2026, 8, day, 9, 0)
  const task = tasks.find((candidate) => candidate.id === taskId)!
  return {
    id: id as TimeEntryId,
    taskId,
    taskLabel: task.label,
    taskTitle: task.title,
    projectLabel: task.projectLabel,
    anchor: task.anchor,
    startedAt: started.toISOString(),
    stoppedAt: new Date(started.getTime() + hours * 3600 * 1000).toISOString(),
    createdAt: started.toISOString(),
    updatedAt: started.toISOString()
  }
}

/** Typed with its argument, so the assertion below reads the text that was copied. */
const writeText = vi.fn(async (text: string) => void text)

function chipFor(wrapper: VueWrapper, name: string): DOMWrapper<Element> {
  const chip = wrapper
    .findAll('[data-testid="report-company-chip"]')
    .find((candidate) => candidate.text().includes(name))
  if (!chip) throw new Error(`No chip for ${name}`)
  return chip
}

async function mountReport() {
  setActivePinia(createPinia())
  const store = useConsoleStore()
  store.companies = companies
  store.tasks = tasks
  // 2h on Tuesday for acme, 3h on Wednesday for globex.
  store.timeEntries = [session('te1', builder, 1, 2), session('te2', signal, 2, 3)]
  await router.push('/report')
  const wrapper = mount(ReportPage, { global: { plugins: [router] } })
  await flushPromises()
  return wrapper
}

describe('ReportPage', () => {
  beforeEach(() => {
    vi.useFakeTimers({ shouldAdvanceTime: true })
    vi.setSystemTime(TODAY)
    writeText.mockClear()
    Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText } })
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('opens on the week in progress, totalled across every company', async () => {
    const wrapper = await mountReport()

    expect(wrapper.get('[data-testid="report-total"]').text()).toBe('5h')
    expect(wrapper.findAll('[data-testid="report-company"]')).toHaveLength(2)
    expect(wrapper.findAll('[data-testid="report-task"]')).toHaveLength(2)
    // Seven columns, one per day, whether or not anything ran on them.
    expect(wrapper.findAll('[data-testid="ridge-column"]')).toHaveLength(7)
  })

  it('narrows to the companies picked, and back to all of them', async () => {
    const wrapper = await mountReport()

    // By name, not by position: the chips are ordered by how much time each company has.
    await chipFor(wrapper, 'globex').trigger('click')

    expect(wrapper.get('[data-testid="report-total"]').text()).toBe('3h')
    expect(wrapper.findAll('[data-testid="report-company"]')).toHaveLength(1)
    expect(wrapper.get('[data-testid="report-company"]').text()).toContain('globex')

    await wrapper.get('[data-testid="report-clear-filter"]').trigger('click')

    expect(wrapper.get('[data-testid="report-total"]').text()).toBe('5h')
  })

  /** The chips are built from the whole period, so filtering never removes the way back. */
  it('keeps every company in the filter while one of them is picked', async () => {
    const wrapper = await mountReport()

    await wrapper.findAll('[data-testid="report-company-chip"]')[0]!.trigger('click')

    expect(wrapper.findAll('[data-testid="report-company-chip"]')).toHaveLength(2)
  })

  it('steps to another week and says plainly that nothing was tracked in it', async () => {
    const wrapper = await mountReport()

    await wrapper.get('[data-testid="report-prev"]').trigger('click')

    expect(wrapper.get('[data-testid="report-total"]').text()).toBe('0m')
    expect(wrapper.get('[data-testid="empty-state"]').text()).toContain('Nothing tracked')
    // The way back to now only appears once there is somewhere to go back from.
    expect(wrapper.find('[data-testid="report-current"]').exists()).toBe(true)
  })

  it('reframes the same sessions as a month without moving off them', async () => {
    const wrapper = await mountReport()

    await wrapper.get('[data-testid="report-period-month"]').trigger('click')

    expect(wrapper.get('[data-testid="report-total"]').text()).toBe('5h')
    expect(wrapper.findAll('[data-testid="ridge-column"]')).toHaveLength(30)
  })

  it('hands the report over as markdown, anchors and all', async () => {
    const wrapper = await mountReport()

    await wrapper.get('[data-testid="report-copy"]').trigger('click')
    await flushPromises()

    const markdown = writeText.mock.calls[0]![0]
    expect(markdown).toContain('## acme · 2h')
    expect(markdown).toContain('`project:beacon task:signal-ingest`')
  })
})
