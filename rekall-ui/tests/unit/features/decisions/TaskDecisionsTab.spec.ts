import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import * as api from '@/features/decisions/api/decisions.api'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import TaskDecisionsTab from '@/features/task/components/TaskDecisionsTab.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { Task } from '@/features/catalog/model/catalog'
import { answered, decision, TASK } from './fixtures'

vi.mock('@/features/decisions/api/decisions.api')

const task = { id: TASK, code: 'VG-41', anchor: 'task:VG-41', projectRepoFolder: '/code/vega' } as unknown as Task

function mountTab(forTask: Task = task) {
  return mount(TaskDecisionsTab, { props: { task: forTask } })
}

async function settle(): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve))
}

describe('the Decisions tab', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  it('says nothing is recorded yet and how to start, with no ledger and no switch', async () => {
    vi.mocked(api.fetchDecisions).mockResolvedValue({ decisions: [], sessionBlock: '' })
    const page = mountTab()
    await settle()

    expect(page.text()).toContain('Nothing recorded yet')
    expect(page.text()).toContain('/rk task:VG-41 refine')
    expect(page.find('[data-testid="decision-ledger"]').exists()).toBe(false)
    expect(page.find('[data-testid="segment-read"]').exists()).toBe(false)
  })

  it('lists the decisions grouped by topic with where they stand, and loads them when the task is shown', async () => {
    vi.mocked(api.fetchDecisions).mockResolvedValue({
      decisions: [answered('namespace', 'billing', { topic: 'Cluster' }), decision('db-password', { topic: 'Secrets' })],
      sessionBlock: '<decisions/>'
    })
    const page = mountTab()
    await settle()

    expect(api.fetchDecisions).toHaveBeenCalledWith(TASK)
    expect(page.findAll('[data-testid="decision-group"]').map((group) => group.find('h2').text())).toEqual(['Secrets', 'Cluster'])
    expect(page.get('[data-testid="decisions-tally"]').text()).toContain('1 question still open.')
    expect(useConsoleStore().decisionsByTask[TASK]?.decisions).toHaveLength(2)
  })

  it('switches to the text a session reads, as the server rendered it', async () => {
    vi.mocked(api.fetchDecisions).mockResolvedValue({ decisions: [answered('region', 'eu-west-1')], sessionBlock: '<decisions count="1">\nregion: eu-west-1\n</decisions>' })
    const page = mountTab()
    await settle()

    await page.get('[data-testid="segment-read"]').trigger('click')

    expect(page.get('[data-testid="decision-block"]').text()).toBe('<decisions count="1">\nregion: eu-west-1\n</decisions>')
    expect(page.find('[data-testid="decision-ledger"]').exists()).toBe(false)
  })

  it('has a session refine the task from the button, whether or not decisions were recorded', async () => {
    vi.mocked(api.fetchDecisions).mockResolvedValue({ decisions: [], sessionBlock: '' })
    const planForTask = vi.spyOn(useTerminalStore(), 'planForTask').mockResolvedValue({} as never)
    const page = mountTab()
    await settle()

    await page.get('[data-testid="decision-refine"]').trigger('click')

    expect(planForTask).toHaveBeenCalledWith(TASK, 'task:VG-41', expect.anything(), 'REFINE', '')
  })

  it('refuses to start a session when the project has no folder', async () => {
    vi.mocked(api.fetchDecisions).mockResolvedValue({ decisions: [], sessionBlock: '' })
    const planForTask = vi.spyOn(useTerminalStore(), 'planForTask').mockResolvedValue({} as never)
    const page = mountTab({ ...task, projectRepoFolder: null } as Task)
    await settle()

    await page.get('[data-testid="decision-refine"]').trigger('click')

    expect(planForTask).not.toHaveBeenCalled()
  })
})
