import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount, RouterLinkStub } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import * as api from '@/features/decisions/api/decisions.api'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { setPreferredProvider } from '@/common/config/session-launch'
import TaskDecisionsTab from '@/features/task/components/TaskDecisionsTab.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { Terminal } from '@/features/claude/model/terminal'
import type { Task } from '@/features/catalog/model/catalog'
import { answered, decision, TASK } from './fixtures'
import { installStorage } from '../../support/memoryStorage'

vi.mock('@/features/decisions/api/decisions.api')
vi.mock('vue-router', () => ({ useRouter: () => ({}) }))

const task = { id: TASK, code: 'VG-41', anchor: 'task:VG-41', projectRepoFolder: '/code/vega' } as unknown as Task

function mountTab(forTask: Task = task) {
  return mount(TaskDecisionsTab, { props: { task: forTask }, global: { stubs: { RouterLink: RouterLinkStub } } })
}

async function settle(): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve))
}

async function refine(page: ReturnType<typeof mountTab>): Promise<void> {
  await page.get('[data-testid="decision-refine"]').trigger('click')
  await flushPromises()
}

describe('the Decisions tab', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
    installStorage()
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

  it('searches the decisions, lists the ones that contain the string and counts them', async () => {
    vi.mocked(api.fetchDecisions).mockResolvedValue({
      decisions: [
        answered('region', 'eu-west-1', { topic: 'Cluster' }),
        answered('namespace', 'billing', { topic: 'Cluster' }),
        decision('db-password', { topic: 'Secrets' })
      ],
      sessionBlock: '<decisions/>'
    })
    const page = mountTab()
    await settle()

    await page.get('[data-testid="decision-search"]').setValue('BILL')

    expect(page.findAll('[data-testid="decision-row"]').map((row) => row.attributes('data-key'))).toEqual(['namespace'])
    expect(page.findAll('[data-testid="decision-group"]')).toHaveLength(1)
    expect(page.get('[data-testid="decision-group-count"]').text()).toBe('1 of 2 found')
    expect(page.get('[data-testid="decision-search-count"]').text()).toBe('1 of 3 decisions')
    expect(page.get('mark').text()).toBe('bill')
  })

  it('says no decision contains the string and clears the search from there', async () => {
    vi.mocked(api.fetchDecisions).mockResolvedValue({ decisions: [answered('region', 'eu-west-1')], sessionBlock: '<decisions/>' })
    const page = mountTab()
    await settle()
    await page.get('[data-testid="decision-search"]').setValue('zeppelin')

    expect(page.get('[data-testid="decision-none"]').text()).toContain('No decision contains “zeppelin”')

    await page.get('[data-testid="decision-none-clear"]').trigger('click')

    expect(page.findAll('[data-testid="decision-row"]')).toHaveLength(1)
    expect(page.get<HTMLInputElement>('[data-testid="decision-search"]').element.value).toBe('')
    expect(page.get('[data-testid="decision-search-count"]').text()).toBe('')
  })

  it('hides the search over the session text and starts empty on another task', async () => {
    vi.mocked(api.fetchDecisions).mockResolvedValue({ decisions: [answered('region', 'eu-west-1')], sessionBlock: '<decisions/>' })
    const page = mountTab()
    await settle()
    await page.get('[data-testid="decision-search"]').setValue('eu')

    await page.get('[data-testid="segment-read"]').trigger('click')
    expect(page.find('[data-testid="decision-search"]').exists()).toBe(false)

    await page.get('[data-testid="segment-ledger"]').trigger('click')
    expect(page.get<HTMLInputElement>('[data-testid="decision-search"]').element.value).toBe('eu')

    await page.setProps({ task: { ...task, id: 't-2' } as Task })
    await settle()
    await useConsoleStore().loadDecisions('t-2' as Task['id'])
    await settle()
    expect(page.get<HTMLInputElement>('[data-testid="decision-search"]').element.value).toBe('')
  })

  it.each(['CLAUDE', 'CODEX'] as const)('refines with %s and shows its answer in the task ledger', async (provider) => {
    setPreferredProvider(provider)
    vi.mocked(api.fetchDecisions).mockResolvedValue({ decisions: [], sessionBlock: '' })
    const planForTask = vi.spyOn(useTerminalStore(), 'planForTask').mockResolvedValue({} as never)
    const page = mountTab()
    await settle()

    await refine(page)

    expect(planForTask).toHaveBeenCalledTimes(1)
    expect(planForTask).toHaveBeenCalledWith(TASK, expect.objectContaining({ provider }), 'REFINE', '')

    vi.mocked(api.fetchDecisions).mockResolvedValue({
      decisions: [answered('region', 'eu-west-1')],
      sessionBlock: '<decisions/>'
    })
    await useConsoleStore().loadDecisions(TASK)
    await settle()

    expect(page.get('[data-testid="decision-ledger"]').text()).toContain('eu-west-1')
  })

  it('sends Refine to the task’s live Claude Code session with Codex selected', async () => {
    setPreferredProvider('CODEX')
    vi.mocked(api.fetchDecisions).mockResolvedValue({ decisions: [], sessionBlock: '' })
    const terminals = useTerminalStore()
    terminals.terminals = [{ taskId: TASK, live: true, provider: 'CLAUDE' } as Terminal]
    const planForTask = vi.spyOn(terminals, 'planForTask').mockResolvedValue({} as never)
    const page = mountTab({ ...task, projectRepoFolder: null })

    await refine(page)

    expect(planForTask).toHaveBeenCalledWith(TASK, expect.objectContaining({ provider: 'CLAUDE' }), 'REFINE', '')
  })

  it('refuses to start a session when the project has no folder', async () => {
    vi.mocked(api.fetchDecisions).mockResolvedValue({ decisions: [], sessionBlock: '' })
    const planForTask = vi.spyOn(useTerminalStore(), 'planForTask').mockResolvedValue({} as never)
    const page = mountTab({ ...task, projectRepoFolder: null } as Task)
    await settle()

    await refine(page)

    expect(planForTask).not.toHaveBeenCalled()
  })
})
