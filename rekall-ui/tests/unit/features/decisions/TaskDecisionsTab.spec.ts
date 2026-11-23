import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import * as api from '@/features/decisions/api/decisions.api'
import TaskDecisionsTab from '@/features/task/components/TaskDecisionsTab.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { Task } from '@/features/catalog/model/catalog'
import { answered, decision, TASK } from './fixtures'

vi.mock('@/features/decisions/api/decisions.api')

const task = { id: TASK, code: 'VG-41' } as unknown as Task

function mountTab() {
  return mount(TaskDecisionsTab, { props: { task } })
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
})
