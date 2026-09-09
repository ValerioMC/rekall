import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import NotePane from '@/components/console/NotePane.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { RekallDocument, Task, TaskStatus } from '@/model/catalog'
import type { DocumentId, ProjectId, TaskId } from '@/model/branded'

/**
 * The membership strip. Tasks still in play are always on it; finished ones fold behind a count
 * so a note that turned out to be transversal does not open onto a wall of dead anchors. The
 * write path is `store.saveNote`, stubbed here and covered in `console.spec`.
 */
const rekall = 'p1' as ProjectId

const live = 't1' as TaskId
const alsoLive = 't2' as TaskId
const shipped = 't3' as TaskId
const archived = 't4' as TaskId

const task = (id: TaskId, label: string, title: string, status: TaskStatus): Task => ({
  id,
  label,
  title,
  status,
  description: null,
  autoWrapup: false,
  wrapupDirective: null,
  projectId: rekall,
  projectLabel: 'rekall',
  projectTitle: 'Rekall',
  companyName: 'vforge',
  projectRepoFolder: null,
  documentCount: 0,
  stepCount: 0,
  stepsDone: 0,
  hasWrapup: false,
  reviewState: 'OPEN',
  reviewActive: true,
  claimedAt: null,
  acceptedAt: null,
  reviewNote: null,
  anchor: `project:rekall task:${label}`,
  updatedAt: '2026-09-01T10:00:00Z'
})

const tasks: Task[] = [
  task(live, 'note-refactor', 'Note refactor', 'IN_PROGRESS'),
  task(alsoLive, 'filing-drawer', 'Filing drawer', 'TODO'),
  task(shipped, 'sse-conduit', 'SSE conduit', 'DONE'),
  task(archived, 'old-layout', 'Old layout', 'DONE')
]

const taskRef = (t: Task) => ({
  id: t.id,
  label: t.label,
  title: t.title,
  projectLabel: t.projectLabel,
  projectTitle: t.projectTitle,
  companyName: t.companyName,
  anchor: t.anchor
})

function makeDocument(on: Task[]): RekallDocument {
  return {
    id: 'd1' as DocumentId,
    title: 'conventions.md',
    kind: 'notes',
    bodyMarkdown: 'body',
    tasks: on.map(taskRef),
    updatedAt: '2026-09-05T10:00:00Z'
  }
}

function seed(document: RekallDocument, selectedTaskId: TaskId | null) {
  const store = useConsoleStore()
  store.tasks = tasks
  store.documents = [document]
  store.selectedDocId = document.id
  store.selectedTaskId = selectedTaskId
  store.isLoading = false
  store.saveNote = vi.fn().mockResolvedValue(undefined)
  store.selectTask = vi.fn()
  return store
}

let pinia: Pinia

function render() {
  return mount(NotePane, {
    global: {
      plugins: [pinia],
      stubs: {
        AppMarkdownEditor: true,
        NoteAssignmentDialog: true,
        LaunchClaudeCodeButton: true,
        AppConfirm: true
      }
    }
  })
}

describe('NotePane membership strip', () => {
  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)
  })

  it('shows tasks in play and folds finished ones behind a count', async () => {
    seed(makeDocument([tasks[0]!, tasks[1]!, tasks[2]!, tasks[3]!]), live)
    const wrapper = render()
    await flushPromises()

    const liveChips = wrapper.findAll('[data-testid="note-task-chip"]').map((c) => c.text())
    expect(liveChips.join(' ')).toContain('rekall/note-refactor')
    expect(liveChips.join(' ')).toContain('rekall/filing-drawer')

    expect(wrapper.findAll('[data-testid="note-task-chip-done"]')).toHaveLength(0)
    expect(wrapper.get('[data-testid="note-done-toggle"]').text()).toContain('2 done')
    expect(wrapper.text()).toContain('On 4 tasks')
  })

  it('reveals the finished chips when the count is clicked', async () => {
    seed(makeDocument([tasks[0]!, tasks[2]!, tasks[3]!]), live)
    const wrapper = render()
    await flushPromises()

    await wrapper.get('[data-testid="note-done-toggle"]').trigger('click')

    const doneChips = wrapper.findAll('[data-testid="note-task-chip-done"]').map((c) => c.text())
    expect(doneChips.join(' ')).toContain('rekall/sse-conduit')
    expect(doneChips.join(' ')).toContain('rekall/old-layout')
  })

  it('shows the finished chips outright when every task the note is on is done', async () => {
    seed(makeDocument([tasks[2]!, tasks[3]!]), shipped)
    const wrapper = render()
    await flushPromises()

    expect(wrapper.findAll('[data-testid="note-task-chip-done"]')).toHaveLength(2)
    expect(wrapper.find('[data-testid="note-done-toggle"]').exists()).toBe(false)
  })

  it('detaches a task from its chip, keeping the rest', async () => {
    const store = seed(makeDocument([tasks[0]!, tasks[1]!]), live)
    const wrapper = render()
    await flushPromises()

    await wrapper.get('[aria-label="Remove this note from Filing drawer"]').trigger('click')

    expect(store.saveNote).toHaveBeenCalledWith('d1', { taskIds: [live] })
  })
})
