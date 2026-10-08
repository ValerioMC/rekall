import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import HomeReviewCard from '@/components/console/home/HomeReviewCard.vue'
import HomeSessionsCard from '@/components/console/home/HomeSessionsCard.vue'
import HomeSituation from '@/components/console/home/HomeSituation.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/stores/terminal.store'
import type { Task, TaskStep } from '@/model/catalog'
import type { Terminal } from '@/model/terminal'
import type { ProjectId, TaskId, TaskStepId, TerminalId } from '@/model/branded'

const report: Task = {
  id: 't1' as TaskId,
  label: 'report-builder',
  title: 'Report builder',
  status: 'IN_PROGRESS',
  description: null,
  projectId: 'p1' as ProjectId,
  projectLabel: 'vega',
  projectTitle: 'Vega',
  companyName: 'acme',
  projectRepoFolder: null,
  documentCount: 0,
  stepCount: 1,
  stepsDone: 0,
  draftStepCount: 0,
  hasWrapup: false,
  reviewState: 'OPEN',
  reviewActive: false,
  claimedAt: null,
  acceptedAt: null,
  reviewNote: null,
  tagId: null,
  tagName: null,
  tagIcon: null,
  tagColor: null,
  anchor: 'project:vega task:report-builder',
  updatedAt: '2026-09-22T09:00:00Z'
}

const claimedStep: TaskStep = {
  id: 's1' as TaskStepId,
  taskId: report.id,
  title: 'Wire the export',
  bodyMarkdown: null,
  state: 'CLAIMED',
  done: false,
  runningAt: null,
  claimedAt: '2026-09-22T09:00:00Z',
  doneAt: null,
  position: 0,
  passes: [],
  createdAt: '2026-09-22T08:00:00Z',
  updatedAt: '2026-09-22T09:00:00Z'
}

const session: Terminal = {
  id: 'term1' as TerminalId,
  taskId: report.id,
  stepId: null,
  anchors: 'project:vega task:report-builder',
  workingDir: '/work/vega',
  projectLabel: 'vega',
  taskLabel: 'report-builder',
  taskTitle: 'Report builder',
  skipPermissions: false,
  model: null,
  effort: null,
  live: true,
  startedAt: new Date(Date.now() - 600_000).toISOString(),
  lastActivityAt: new Date().toISOString()
}

describe('the console home', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('says what runs and lights what waits for review', () => {
    const store = useConsoleStore()
    store.tasks = [report]
    store.steps = [claimedStep]
    useTerminalStore().terminals = [session]

    const wrapper = mount(HomeSituation)

    expect(wrapper.text()).toContain('1 session working,')
    const waiting = wrapper.get('[data-testid="home-situation-waiting"]')
    expect(waiting.text()).toBe('1 waiting for your review.')
    expect(waiting.classes()).toContain('text-accent-strong')
  })

  it('lists claimed work and opens the one picked', async () => {
    const store = useConsoleStore()
    store.tasks = [report]
    store.steps = [claimedStep]
    store.openReviewItem = vi.fn()

    const wrapper = mount(HomeReviewCard)
    const row = wrapper.get('[data-testid="home-review-item"]')
    expect(row.text()).toContain('Wire the export')
    expect(row.text()).toContain('project:vega task:report-builder')

    await row.trigger('click')
    expect(store.openReviewItem).toHaveBeenCalledWith(expect.objectContaining({ stepId: 's1' }))
  })

  it('explains the review card when nothing waits', () => {
    const wrapper = mount(HomeReviewCard)

    expect(wrapper.find('[data-testid="home-review-item"]').exists()).toBe(false)
    expect(wrapper.text()).toContain('Nothing to review')
  })

  it('opens a running session on its terminal', async () => {
    const store = useConsoleStore()
    const terminals = useTerminalStore()
    terminals.terminals = [session, { ...session, id: 'term2' as TerminalId, live: false }]
    store.selectTask = vi.fn()
    store.openTerminal = vi.fn()

    const wrapper = mount(HomeSessionsCard)
    const rows = wrapper.findAll('[data-testid="home-session"]')
    expect(rows).toHaveLength(1)
    expect(rows[0]!.text()).toContain('10m')

    await rows[0]!.trigger('click')
    expect(store.selectTask).toHaveBeenCalledWith('t1')
    expect(store.openTerminal).toHaveBeenCalled()
    expect(terminals.activeTerminalId).toBe('term1')
  })
})
