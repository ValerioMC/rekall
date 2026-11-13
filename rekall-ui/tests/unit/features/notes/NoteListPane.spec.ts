import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { mount } from '@vue/test-utils'
import NoteListPane from '@/features/notes/components/NoteListPane.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { RekallDocument } from '@/features/notes/model/document'
import type { Task } from '@/features/catalog/model/catalog'
import type { ProjectId, TaskId } from '@/model/branded'

/** The task's column: the dossier of cards above the Information list, which `InformationList.spec` covers. */
const vega = 'p1' as ProjectId
const builder = 't1' as TaskId
const retry = 't2' as TaskId

const task = (id: TaskId, label: string, title: string): Task => ({
  id,
  code: label,
  label,
  title,
  status: 'IN_PROGRESS',
  description: null,
  projectId: vega,
  projectLabel: 'vega',
  projectTitle: 'Vega Platform',
  companyName: 'acme',
  projectRepoFolder: null,
  documentCount: 0,
  stepCount: 0,
  stepsDone: 0,
  draftStepCount: 0,
  hasWrapup: false,
  reviewState: 'OPEN',
  reviewActive: false,
  claimedAt: null,
  acceptedAt: null,
  tagId: null,
  tagName: null,
  tagIcon: null,
  tagColor: null,
  graphLaidAt: null,
  releaseSkipped: false,
  kind: 'TASK',
  epicId: null, quick: false, autoAccepted: false, dependsOn: [], trackerIssueKey: null, trackerIssueUrl: null, trackerSyncError: null, trackerContentSynced: true, pullRequestNumber: null, pullRequestUrl: null, pullRequestFailure: null, pullRequestMergedAt: null,
  anchor: `project:vega task:${label}`,
  updatedAt: '2026-09-01T10:00:00Z'
})

const tasks: Task[] = [
  task(builder, 'report-builder', 'Report builder'),
  task(retry, 'retry-policy', 'Retry policy')
]

let pinia: Pinia

function seed(documents: RekallDocument[]) {
  const store = useConsoleStore()
  store.tasks = tasks
  store.documents = documents
  store.selectedTaskId = builder
  store.selectedDocId = documents[0]?.id ?? null
  store.isLoading = false
  store.deleteNote = vi.fn().mockResolvedValue(undefined)
  store.selectDocument = vi.fn()
  return store
}

function render() {
  return mount(NoteListPane, { attachTo: document.body, global: { plugins: [pinia] } })
}


describe('NoteListPane on an epic', () => {
  const epic = 'e1' as TaskId

  beforeEach(() => {
    document.body.innerHTML = ''
    pinia = createPinia()
    setActivePinia(pinia)
  })

  /** Its tasks and its description are one section in the console, so the dossier offers one way in. */
  it('shows one card for its tasks and description, with how many are done, and opens the epic pane', async () => {
    const store = seed([])
    store.tasks = [
      { ...task(epic, 'epics', 'Epics'), kind: 'EPIC' },
      { ...task(builder, 'report-builder', 'Report builder'), epicId: epic, quick: false, autoAccepted: false, status: 'DONE' },
      { ...task(retry, 'retry-policy', 'Retry policy'), epicId: epic }
    ]
    store.selectedTaskId = epic
    const wrapper = render()

    expect(wrapper.find('[data-testid="description-card"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="steps-card"]').exists()).toBe(false)
    expect(wrapper.get('[data-testid="epic-dossier-progress"]').text()).toBe('1/2')

    await wrapper.get('[data-testid="epic-dossier-card"]').trigger('click')
    expect(store.paneFocus).toBe('description')
    wrapper.unmount()
  })

  it("opens a task's epic from the card above its dossier", async () => {
    const store = seed([])
    store.tasks = [{ ...task(epic, 'epics', 'Epics'), kind: 'EPIC' }, { ...task(builder, 'report-builder', 'Report builder'), epicId: epic }]
    const wrapper = render()

    await wrapper.get('[data-testid="epic-membership"]').trigger('click')

    expect(store.selectedTaskId).toBe(epic)
    expect(store.paneFocus).toBe('description')
    wrapper.unmount()
  })
})
