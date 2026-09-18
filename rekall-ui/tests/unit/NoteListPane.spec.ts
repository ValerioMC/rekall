import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import NoteListPane from '@/components/console/NoteListPane.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'
import type { RekallDocument, Task, TaskRef } from '@/model/catalog'
import type { DocumentId, ProjectId, TaskId } from '@/model/branded'

/**
 * The task's note column: each card carries its note's membership at the right end, and that
 * slot is how a note is taken off the task in view. Only the choice of what to detach, the
 * delete confirm on a note that is only here and the Undo that follows are decided here; the
 * write path is the store's, stubbed and covered in `console.spec`.
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
  reviewNote: null,
  tagId: null,
  tagName: null,
  tagIcon: null,
  tagColor: null,
  anchor: `project:vega task:${label}`,
  updatedAt: '2026-09-01T10:00:00Z'
})

const tasks: Task[] = [
  task(builder, 'report-builder', 'Report builder'),
  task(retry, 'retry-policy', 'Retry policy')
]

const refOf = (id: TaskId): TaskRef => {
  const found = tasks.find((candidate) => candidate.id === id)!
  return {
    id,
    label: found.label,
    title: found.title,
    projectLabel: found.projectLabel,
    projectTitle: found.projectTitle,
    companyName: found.companyName,
    anchor: found.anchor
  }
}

const note = (id: string, title: string, on: TaskId[]): RekallDocument => ({
  id: id as DocumentId,
  title,
  kind: 'notes',
  bodyMarkdown: 'Accesso via bastion',
  tasks: on.map(refOf),
  updatedAt: '2026-09-01T10:00:00Z'
})

let pinia: Pinia

function seed(documents: RekallDocument[]) {
  const store = useConsoleStore()
  store.tasks = tasks
  store.documents = documents
  store.selectedTaskId = builder
  store.selectedDocId = documents[0]?.id ?? null
  store.navMode = 'tasks'
  store.isLoading = false
  store.attachNoteToTask = vi.fn().mockResolvedValue(undefined)
  store.detachNoteFromTask = vi.fn().mockResolvedValue(undefined)
  store.deleteNote = vi.fn().mockResolvedValue(undefined)
  store.selectDocument = vi.fn()
  return store
}

function render() {
  return mount(NoteListPane, { attachTo: document.body, global: { plugins: [pinia] } })
}

const cardsOf = (wrapper: ReturnType<typeof render>) => wrapper.findAll('[data-testid="note-card"]')

describe('NoteListPane', () => {
  beforeEach(() => {
    document.body.innerHTML = ''
    pinia = createPinia()
    setActivePinia(pinia)
  })

  it('says where else each note lives, and names the way off as delete when there is nowhere else', () => {
    seed([note('d1', 'cluster.md', [builder, retry]), note('d2', 'conventions.md', [builder])])
    const wrapper = render()

    const membership = wrapper.findAll('[data-testid="note-card-membership"]')
    expect(membership).toHaveLength(2)

    expect(membership[0]!.find('[data-testid="note-card-elsewhere"]').text()).toContain('2')
    expect(membership[0]!.find('[data-testid="note-card-off"]').exists()).toBe(true)
    expect(membership[0]!.find('[data-testid="note-card-delete"]').exists()).toBe(false)

    expect(membership[1]!.find('[data-testid="note-card-elsewhere"]').exists()).toBe(false)
    expect(membership[1]!.find('[data-testid="note-card-off"]').exists()).toBe(false)
    expect(membership[1]!.find('[data-testid="note-card-delete"]').attributes('title')).toContain(
      'deletes the note'
    )
    wrapper.unmount()
  })

  it('takes a note off the task in view with one click, without opening the note', async () => {
    const store = seed([note('d1', 'cluster.md', [builder, retry])])
    const wrapper = render()

    await wrapper.find('[data-testid="note-card-off"]').trigger('click')
    await flushPromises()

    expect(store.detachNoteFromTask).toHaveBeenCalledWith('d1', builder)
    expect(store.selectDocument).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('takes a note off the task from the keyboard, with Backspace or Delete on its card', async () => {
    const store = seed([note('d1', 'cluster.md', [builder, retry])])
    const wrapper = render()

    await cardsOf(wrapper)[0]!.trigger('keydown', { key: 'Backspace' })
    await flushPromises()

    expect(store.detachNoteFromTask).toHaveBeenCalledWith('d1', builder)
    wrapper.unmount()
  })

  /**
   * A note is on at least one task, so taking it off the only one is deleting it: the control
   * and the keyboard both open the confirm, and nothing is written until it is answered.
   */
  it('asks before deleting a note that is only here, from the control or the keyboard', async () => {
    const store = seed([note('d1', 'cluster.md', [builder])])
    const wrapper = render()

    await cardsOf(wrapper)[0]!.trigger('keydown', { key: 'Delete' })
    await flushPromises()

    const confirm = document.body.querySelector('[role="alertdialog"]')
    expect(confirm?.getAttribute('aria-label')).toBe('Delete cluster.md?')
    expect(confirm?.textContent).toContain('only task the note is on')
    expect(store.detachNoteFromTask).not.toHaveBeenCalled()
    expect(store.deleteNote).not.toHaveBeenCalled()

    const keep = Array.from(confirm!.querySelectorAll('button')).find((b) => b.textContent?.includes('Keep it'))!
    keep.click()
    await flushPromises()
    expect(document.body.querySelector('[role="alertdialog"]')).toBeNull()

    await wrapper.find('[data-testid="note-card-delete"]').trigger('click')
    await flushPromises()
    const again = document.body.querySelector('[role="alertdialog"]')!
    const del = Array.from(again.querySelectorAll('button')).find((b) => b.textContent?.includes('Delete note'))!
    del.click()
    await flushPromises()

    expect(store.deleteNote).toHaveBeenCalledWith('d1')
    expect(store.detachNoteFromTask).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  /** No confirm before taking a note off: the toast that follows offers Undo instead. */
  it('offers Undo in the toast, which puts the note back and reopens it if it was open', async () => {
    const store = seed([note('d1', 'cluster.md', [builder, retry])])
    const wrapper = render()

    await wrapper.find('[data-testid="note-card-off"]').trigger('click')
    await flushPromises()

    const toast = useToastStore()
    const announced = toast.toasts[0]
    expect(announced?.message).toBe('cluster.md is off this task.')
    expect(announced?.action?.label).toBe('Undo')

    toast.act(announced!.id)
    await flushPromises()

    expect(store.attachNoteToTask).toHaveBeenCalledWith('d1', builder)
    expect(store.selectDocument).toHaveBeenCalledWith('d1')
    expect(toast.toasts).toHaveLength(0)
    wrapper.unmount()
  })

  it('does not offer to reopen a note that was not the one in the editor', async () => {
    const store = seed([note('d1', 'cluster.md', [builder, retry]), note('d2', 'other.md', [builder, retry])])
    const wrapper = render()

    await wrapper.findAll('[data-testid="note-card-off"]')[1]!.trigger('click')
    await flushPromises()

    const toast = useToastStore()
    toast.act(toast.toasts[0]!.id)
    await flushPromises()

    expect(store.attachNoteToTask).toHaveBeenCalledWith('d2', builder)
    expect(store.selectDocument).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('says nothing and puts nothing back when the removal failed', async () => {
    const store = seed([note('d1', 'cluster.md', [builder, retry])])
    store.detachNoteFromTask = vi.fn().mockRejectedValue(new Error('offline'))
    const wrapper = render()

    await wrapper.find('[data-testid="note-card-off"]').trigger('click')
    await flushPromises()

    const toast = useToastStore()
    expect(toast.toasts).toHaveLength(1)
    expect(toast.toasts[0]?.kind).toBe('error')
    expect(toast.toasts[0]?.action).toBeNull()
    wrapper.unmount()
  })
})
