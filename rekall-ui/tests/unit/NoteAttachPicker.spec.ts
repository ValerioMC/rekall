import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import NoteAttachPicker from '@/components/console/NoteAttachPicker.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { Project, RekallDocument, Task, TaskRef } from '@/model/catalog'
import type { NoteScope } from '@/model/note-scope'
import type { CompanyId, DocumentId, ProjectId, TaskId } from '@/model/branded'

/**
 * The picker hung under "Add existing note" in the notes column. It offers only notes the task
 * may carry and does not, and attaches one through `store.attachNoteToTask`, stubbed here.
 */
const here = 't1' as TaskId
const elsewhere = 't2' as TaskId
const third = 't3' as TaskId

const ref = (id: TaskId, label: string): TaskRef => ({
  id,
  label,
  title: label,
  projectLabel: 'rekall',
  projectTitle: 'Rekall',
  companyName: 'vforge',
  anchor: `project:rekall task:${label}`
})

const refs: Record<TaskId, TaskRef> = {
  [here]: ref(here, 'note-improvement'),
  [elsewhere]: ref(elsewhere, 'filing-drawer'),
  [third]: ref(third, 'sse-conduit')
}

const rekall = 'p1' as ProjectId
const vforge = 'c1' as CompanyId

const project: Project = {
  id: rekall,
  label: 'rekall',
  title: 'Rekall',
  status: 'ACTIVE',
  icon: 'folder',
  description: null,
  blueprintMarkdown: null,
  repoFolder: null,
  autoCommit: false,
  companyId: vforge,
  companyName: 'vforge',
  taskCount: 3,
  anchor: 'project:rekall',
  updatedAt: '2026-09-01T10:00:00Z'
}

const taskHere: Task = {
  id: here,
  label: 'note-improvement',
  title: 'note-improvement',
  status: 'IN_PROGRESS',
  description: null,
  projectId: rekall,
  projectLabel: 'rekall',
  projectTitle: 'Rekall',
  companyName: 'vforge',
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
  anchor: 'project:rekall task:note-improvement',
  updatedAt: '2026-09-01T10:00:00Z'
}

const doc = (
  id: string,
  title: string,
  on: TaskId[],
  updatedAt = '2026-09-01T10:00:00Z',
  scope: NoteScope = { kind: 'GLOBAL' }
): RekallDocument => ({
  id: id as DocumentId,
  title,
  kind: 'notes',
  bodyMarkdown: `${title} body`,
  tasks: on.map((taskId) => refs[taskId]!),
  contextMode: 'FULL',
  scope,
  anchor: 'note:00000000',
  updatedAt
})

let pinia: Pinia

function seed(documents: RekallDocument[]) {
  const store = useConsoleStore()
  store.documents = documents
  store.projects = [project]
  store.tasks = [taskHere]
  store.selectedTaskId = here
  store.isLoading = false
  store.attachNoteToTask = vi.fn().mockResolvedValue(undefined)
  store.selectDocument = vi.fn()
  return store
}

function render() {
  const anchor = document.createElement('div')
  document.body.appendChild(anchor)
  return mount(NoteAttachPicker, {
    props: { taskId: here, anchor },
    attachTo: document.body,
    global: { plugins: [pinia] }
  })
}

const rowsOf = () =>
  Array.from(document.body.querySelectorAll<HTMLElement>('[data-testid="note-attach-row"]'))

describe('NoteAttachPicker', () => {
  beforeEach(() => {
    document.body.innerHTML = ''
    pinia = createPinia()
    setActivePinia(pinia)
  })

  it('lists only the notes the task may carry and does not carry yet, newest first', () => {
    seed([
      doc('d1', 'on-here.md', [here]),
      doc('d2', 'old.md', [elsewhere], '2026-08-01T10:00:00Z'),
      doc('d3', 'fresh.md', [], '2026-09-02T10:00:00Z'),
      doc('d4', 'foreign.md', [], '2026-09-03T10:00:00Z', { kind: 'PROJECT', id: 'other' as ProjectId })
    ])
    const wrapper = render()

    expect(rowsOf().map((row) => row.querySelector('span span')?.textContent?.trim())).toEqual([
      'fresh.md',
      'old.md'
    ])
    expect(document.body.textContent).toContain('Global, on no task')
    expect(document.body.textContent).toContain('Global, on rekall/filing-drawer')
    wrapper.unmount()
  })

  it('filters by title and body', async () => {
    seed([doc('d2', 'runbook.md', []), doc('d3', 'glossary.md', [])])
    const wrapper = render()

    const field = document.body.querySelector<HTMLInputElement>('[data-testid="note-attach-filter"]')!
    field.value = 'gloss'
    field.dispatchEvent(new Event('input'))
    await flushPromises()

    expect(rowsOf()).toHaveLength(1)
    field.value = 'zzz'
    field.dispatchEvent(new Event('input'))
    await flushPromises()
    expect(document.body.querySelector('[data-testid="note-attach-empty"]')?.textContent).toContain('No note matches')
    wrapper.unmount()
  })

  it('attaches the clicked note, opens it and closes', async () => {
    const store = seed([doc('d2', 'runbook.md', [elsewhere])])
    const wrapper = render()

    rowsOf()[0]!.click()
    await flushPromises()

    expect(store.attachNoteToTask).toHaveBeenCalledWith('d2', here)
    expect(store.selectDocument).toHaveBeenCalledWith('d2')
    expect(wrapper.emitted('close')).toHaveLength(1)
    wrapper.unmount()
  })

  it('attaches the highlighted note on Enter and moves with the arrows', async () => {
    const store = seed([doc('d2', 'a.md', [], '2026-09-02T10:00:00Z'), doc('d3', 'b.md', [], '2026-09-01T10:00:00Z')])
    const wrapper = render()

    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown' }))
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter' }))
    await flushPromises()

    expect(store.attachNoteToTask).toHaveBeenCalledWith('d3', here)
    wrapper.unmount()
  })

  it('stays open when the write fails', async () => {
    const store = seed([doc('d2', 'runbook.md', [])])
    store.attachNoteToTask = vi.fn().mockRejectedValue(new Error('offline'))
    const wrapper = render()

    rowsOf()[0]!.click()
    await flushPromises()

    expect(store.selectDocument).not.toHaveBeenCalled()
    expect(wrapper.emitted('close')).toBeUndefined()
    wrapper.unmount()
  })

  it('closes on Escape and says so when nothing is left to add', async () => {
    seed([doc('d1', 'on-here.md', [here])])
    const wrapper = render()

    expect(document.body.querySelector('[data-testid="note-attach-empty"]')?.textContent).toContain('already on it')
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    expect(wrapper.emitted('close')).toHaveLength(1)
    wrapper.unmount()
  })
})
