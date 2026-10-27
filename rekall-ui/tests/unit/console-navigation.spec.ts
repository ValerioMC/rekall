import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'
import type { RekallDocument, TaskRef } from '@/features/catalog/model/catalog'
import type { SearchHit } from '@/features/search/model/search'
import type { ReviewItem } from '@/features/catalog/model/review'
import type { DocumentId, TaskId, TaskStepId } from '@/model/branded'
import { taskFixture } from './support/taskFixture'

/**
 * Where search hits and review items land. Both open something another pane owns, so what is
 * checked here is the pane the store brings forward and the request it leaves for that pane.
 */
const report = 't1' as TaskId
const retry = 't2' as TaskId

const ref = (id: TaskId): TaskRef => ({
  id,
  label: id,
  title: id,
  projectLabel: 'vega',
  projectTitle: 'Vega',
  companyName: 'acme',
  anchor: `project:vega task:${id}`
})

const note: RekallDocument = {
  id: 'd1' as DocumentId,
  title: 'cluster.md',
  kind: 'notes',
  bodyMarkdown: 'bastion',
  tasks: [ref(retry)],
  contextMode: 'FULL',
  scope: { kind: 'GLOBAL' },
  anchor: 'note:00000000',
  updatedAt: '2026-09-01T10:00:00Z'
}

const hit = (kind: SearchHit['kind'], extra: Partial<SearchHit> = {}): SearchHit => ({
  kind,
  taskId: report,
  stepId: null,
  documentId: null,
  title: 'Report builder',
  where: 'project:vega task:t1',
  excerpt: '…settlement batch…',
  ...extra
})

describe('opening from search and from the review queue', () => {
  let store: ReturnType<typeof useConsoleStore>

  beforeEach(() => {
    setActivePinia(createPinia())
    store = useConsoleStore()
    store.documents = [note]
  })

  it('a description or wrapup hit opens that task on that pane', () => {
    store.openSearchHit(hit('DESCRIPTION'))
    expect([store.selectedTaskId, store.paneFocus]).toEqual([report, 'description'])

    store.openSearchHit(hit('WRAPUP'))
    expect(store.paneFocus).toBe('wrapup')
  })

  it('a step hit opens the steps pane and asks it for that step', () => {
    store.openSearchHit(hit('STEP', { stepId: 's9' as TaskStepId }))

    expect(store.paneFocus).toBe('steps')
    expect(store.stepToOpen).toBe('s9')
    store.clearStepToOpen()
    expect(store.stepToOpen).toBeNull()
  })

  it('a note hit opens the note, with one of its tasks in view', () => {
    store.openSearchHit(hit('NOTE', { taskId: retry, documentId: note.id }))

    expect(store.selectedDocId).toBe(note.id)
    expect(store.selectedTaskId).toBe(retry)
    expect(store.paneFocus).toBe('note')
  })

  it('a claimed step opens on its steps pane, a stepless task on its description and review bar', () => {
    const item = (stepId: TaskStepId | null): ReviewItem => ({
      key: stepId ?? report,
      taskId: report,
      stepId,
      taskTitle: 'Report builder',
      stepTitle: stepId ? 'Wire it' : null,
      anchor: 'project:vega task:t1',
      claimedAt: null
    })

    store.openReviewItem(item('s1' as TaskStepId))
    expect([store.paneFocus, store.stepToOpen]).toEqual(['steps', 's1'])

    store.openReviewItem(item(null))
    expect(store.paneFocus).toBe('description')
  })
})

describe('going back to a location', () => {
  let store: ReturnType<typeof useConsoleStore>

  beforeEach(() => {
    setActivePinia(createPinia())
    store = useConsoleStore()
    store.tasks = [taskFixture(report), taskFixture(retry)]
    store.documents = [note]
  })

  it('shows the task, note and pane it names, and closes the composer', () => {
    store.openNoteComposer()

    const shown = store.showLocation({ mode: 'notes', taskId: retry, documentId: note.id, pane: 'wrapup' })

    expect(shown).toEqual({ mode: 'notes', taskId: retry, documentId: note.id, pane: 'wrapup' })
    expect([store.navMode, store.selectedTaskId, store.selectedDocId, store.paneFocus]).toEqual([
      'notes',
      retry,
      note.id,
      'wrapup'
    ])
    expect(store.noteComposerOpen).toBe(false)
  })

  it('drops a task and note deleted since, and with them the task pane', () => {
    const shown = store.showLocation({
      mode: 'tasks',
      taskId: 'gone' as TaskId,
      documentId: 'gone' as DocumentId,
      pane: 'steps'
    })

    expect(shown).toEqual({ mode: 'tasks', taskId: null, documentId: null, pane: 'note' })
  })
})

describe('selecting an epic', () => {
  beforeEach(() => setActivePinia(createPinia()))

  /** An epic has no checklist to work: what it is for is its tasks and the description they share. */
  it('opens the epic on its own pane, and a task on its note', () => {
    const store = useConsoleStore()
    const epic = 'e1' as TaskId
    store.tasks = [{ ...taskFixture(epic), kind: 'EPIC' }, taskFixture(report)]

    store.selectTask(epic)
    expect(store.paneFocus).toBe('description')

    store.selectTask(report)
    expect(store.paneFocus).toBe('note')
  })
})
