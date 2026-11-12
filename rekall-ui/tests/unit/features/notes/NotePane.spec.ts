import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import NotePane from '@/features/notes/components/NotePane.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { RekallDocument } from '@/features/notes/model/document'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskStatus } from '@/features/catalog/model/task-status'
import type { DocumentId, ProjectId, TaskId } from '@/model/branded'

/** The note pane's context mode. The write path is `store.saveNote`, stubbed here and covered in `console.spec`. */
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
  projectId: rekall,
  projectLabel: 'rekall',
  projectTitle: 'Rekall',
  companyName: 'vforge',
  projectRepoFolder: null,
  documentCount: 0,
  stepCount: 0,
  stepsDone: 0, draftStepCount: 0,
  hasWrapup: false,
  reviewState: 'OPEN',
  reviewActive: true,
  claimedAt: null,
  acceptedAt: null,
  tagId: null,
  tagName: null,
  tagIcon: null,
  tagColor: null,
  graphLaidAt: null,
  releaseSkipped: false,
  kind: 'TASK',
  epicId: null, quick: false, autoAccepted: false, dependsOn: [], trackerIssueKey: null, trackerIssueUrl: null, trackerSyncError: null, pullRequestNumber: null, pullRequestUrl: null, pullRequestFailure: null, pullRequestMergedAt: null,
  anchor: `project:rekall task:${label}`,
  updatedAt: '2026-09-01T10:00:00Z'
})

const tasks: Task[] = [
  task(live, 'note-refactor', 'Note refactor', 'IN_PROGRESS'),
  task(alsoLive, 'filing-drawer', 'Filing drawer', 'TODO'),
  task(shipped, 'sse-conduit', 'SSE conduit', 'DONE'),
  task(archived, 'old-layout', 'Old layout', 'DONE')
]

function makeDocument(on: Task): RekallDocument {
  return {
    id: 'd1' as DocumentId,
    title: 'conventions.md',
    kind: 'notes',
    bodyMarkdown: 'body',
    taskId: on.id,
    contextMode: 'FULL',
    anchor: 'note:00000000',
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
        LaunchClaudeCodeButton: true,
        AppConfirm: true
      }
    }
  })
}

describe('NotePane context mode', () => {
  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)
  })

  it('sends a note by reference, and back in full, saving only the mode', async () => {
    const store = seed(makeDocument(tasks[0]!), live)
    const wrapper = render()
    await flushPromises()

    expect(wrapper.get('[data-testid="note-context-mode-full"]').attributes('aria-checked')).toBe('true')

    await wrapper.get('[data-testid="note-context-mode-reference"]').trigger('click')
    expect(store.saveNote).toHaveBeenCalledWith('d1', { contextMode: 'REFERENCE' })
  })

  it('does not save when the mode picked is the one it has', async () => {
    const store = seed(makeDocument(tasks[0]!), live)
    const wrapper = render()
    await flushPromises()

    await wrapper.get('[data-testid="note-context-mode-full"]').trigger('click')
    expect(store.saveNote).not.toHaveBeenCalled()
  })
})
