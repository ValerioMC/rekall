import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import InformationList from '@/features/notes/components/InformationList.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { RekallDocument } from '@/features/notes/model/document'
import type { Task } from '@/features/catalog/model/catalog'
import type { DocumentId, ProjectId, TaskId } from '@/model/branded'

/**
 * The Information list of the task in view: one line per note, how each loads, the excerpt on
 * hover or focus, the confirmed delete, and the add row. The write path is the
 * store's, stubbed here and covered in `console.spec`.
 */
const vega = 'p1' as ProjectId
const builder = 't1' as TaskId
const retry = 't2' as TaskId

const task = (id: TaskId, label: string, title: string): Task => ({
  id,
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

const note = (id: string, title: string, on: TaskId): RekallDocument => ({
  id: id as DocumentId,
  title,
  kind: 'notes',
  bodyMarkdown: 'Accesso via bastion',
  taskId: on,
  contextMode: 'FULL',
  anchor: 'note:00000000',
  updatedAt: '2026-09-01T10:00:00Z'
})

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
  return mount(InformationList, { attachTo: document.body, global: { plugins: [pinia] } })
}

const cardsOf = (wrapper: ReturnType<typeof render>) => wrapper.findAll('[data-testid="note-card"]')

describe('InformationList', () => {
  beforeEach(() => {
    document.body.innerHTML = ''
    pinia = createPinia()
    setActivePinia(pinia)
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('renders one row per note of the task, with no excerpt in the row', () => {
    seed([note('d1', 'cluster.md', builder), note('d2', 'conventions.md', builder), note('d3', 'other.md', retry)])
    const wrapper = render()

    const rows = cardsOf(wrapper)
    expect(rows).toHaveLength(2)
    expect(rows[0]!.text()).toContain('cluster.md')
    expect(rows[0]!.text()).not.toContain('Accesso via bastion')
    expect(wrapper.get('[data-testid="information-list"]').text()).toContain('Information')
    wrapper.unmount()
  })

  it('marks a note loaded on request with the hollow marker and says what it means, only while one is on screen', () => {
    const onRequest: RekallDocument = { ...note('d2', 'runbook.md', builder), contextMode: 'REFERENCE' }
    seed([note('d1', 'cluster.md', builder), onRequest])
    const wrapper = render()

    const markers = wrapper.findAll('[data-testid="note-load-marker"]').map((marker) => marker.attributes('data-load-mode'))
    expect(markers).toEqual(['in-full', 'on-request'])
    expect(wrapper.find('[data-testid="dossier-reference-legend"]').exists()).toBe(true)
    wrapper.unmount()

    document.body.innerHTML = ''
    seed([note('d1', 'cluster.md', builder)])
    const inFull = render()
    expect(inFull.find('[data-testid="dossier-reference-legend"]').exists()).toBe(false)
    inFull.unmount()
  })

  it('names the N key when the task has no information yet', () => {
    seed([])
    const wrapper = render()

    expect(cardsOf(wrapper)).toHaveLength(0)
    expect(wrapper.text()).toContain('Press N to add information')
    wrapper.unmount()
  })

  it('shows the excerpt after a short delay on hover, and drops it when the pointer leaves', async () => {
    seed([note('d1', 'cluster.md', builder)])
    const wrapper = render()
    const row = wrapper.get('[data-testid="note-card"]').element.parentElement!

    row.dispatchEvent(new MouseEvent('mouseenter'))
    expect(wrapper.find('[data-testid="note-excerpt"]').exists()).toBe(false)
    vi.advanceTimersByTime(400)
    await flushPromises()
    expect(wrapper.get('[data-testid="note-excerpt"]').text()).toBe('Accesso via bastion')

    row.dispatchEvent(new MouseEvent('mouseleave'))
    await flushPromises()
    expect(wrapper.find('[data-testid="note-excerpt"]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('shows the excerpt on keyboard focus too', async () => {
    seed([note('d1', 'cluster.md', builder)])
    const wrapper = render()

    wrapper.get('[data-testid="note-card"]').element.dispatchEvent(new FocusEvent('focusin', { bubbles: true }))
    vi.advanceTimersByTime(400)
    await flushPromises()

    expect(wrapper.find('[data-testid="note-excerpt"]').exists()).toBe(true)
    wrapper.unmount()
  })

  it('adds information to the task in view from the last row, and not twice while it is being created', async () => {
    const store = seed([note('d1', 'cluster.md', builder)])
    let finish: () => void = () => undefined
    store.createNote = vi.fn().mockReturnValue(new Promise<void>((resolve) => (finish = resolve)))
    const wrapper = render()

    await wrapper.get('[data-testid="information-add"]').trigger('click')
    expect(store.createNote).toHaveBeenCalledWith(builder)
    expect(wrapper.get('[data-testid="information-add"]').attributes('disabled')).toBeDefined()

    await wrapper.get('[data-testid="information-add"]').trigger('click')
    expect(store.createNote).toHaveBeenCalledTimes(1)

    finish()
    await flushPromises()
    expect(wrapper.get('[data-testid="information-add"]').attributes('disabled')).toBeUndefined()
    wrapper.unmount()
  })

  it('offers a delete on every card', () => {
    seed([note('d1', 'cluster.md', builder), note('d2', 'conventions.md', builder)])
    const wrapper = render()

    expect(wrapper.findAll('[data-testid="note-card-delete"]')).toHaveLength(2)
    wrapper.unmount()
  })

  it('asks before deleting a note, and deletes it once confirmed, without opening it', async () => {
    const store = seed([note('d1', 'cluster.md', builder)])
    const wrapper = render()

    await wrapper.get('[data-testid="note-card-delete"]').trigger('click')
    await flushPromises()

    expect(document.body.querySelector('[role="alertdialog"]')).not.toBeNull()
    expect(store.deleteNote).not.toHaveBeenCalled()
    expect(store.selectDocument).not.toHaveBeenCalled()

    const confirm = [...document.body.querySelectorAll('button')].find((button) => button.textContent?.includes('Delete note'))!
    confirm.click()
    await flushPromises()

    expect(store.deleteNote).toHaveBeenCalledWith('d1')
    wrapper.unmount()
  })

  it('asks the same from the keyboard, with Backspace or Delete on its card', async () => {
    const store = seed([note('d1', 'cluster.md', builder)])
    const wrapper = render()

    await cardsOf(wrapper)[0]!.trigger('keydown', { key: 'Delete' })
    await flushPromises()

    expect(document.body.querySelector('[role="alertdialog"]')).not.toBeNull()
    expect(store.deleteNote).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('deletes nothing when the question is cancelled', async () => {
    const store = seed([note('d1', 'cluster.md', builder)])
    const wrapper = render()

    await wrapper.get('[data-testid="note-card-delete"]').trigger('click')
    await flushPromises()
    const cancel = [...document.body.querySelectorAll('button')].find((button) => button.textContent?.includes('Keep it'))!
    cancel.click()
    await flushPromises()

    expect(document.body.querySelector('[role="alertdialog"]')).toBeNull()
    expect(store.deleteNote).not.toHaveBeenCalled()
    wrapper.unmount()
  })
})
