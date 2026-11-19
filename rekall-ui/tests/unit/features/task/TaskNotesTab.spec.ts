import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { createMemoryHistory, createRouter, type Router } from 'vue-router'
import TaskNotesTab from '@/features/task/components/TaskNotesTab.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { Task } from '@/features/catalog/model/catalog'
import type { RekallDocument } from '@/features/notes/model/document'
import type { DocumentId, ProjectId, TaskId } from '@/model/branded'

const stub = { template: '<div />' }
const taskId = 't1' as TaskId

function makeTask(): Task {
  return {
    id: taskId, code: 'RK-1', label: 'one', title: 'Task one', status: 'IN_PROGRESS', description: null,
    projectId: 'p1' as ProjectId, projectLabel: 'vega', projectTitle: 'Vega', companyName: 'Acme', projectRepoFolder: '/work/vega',
    documentCount: 2, stepCount: 0, stepsDone: 0, draftStepCount: 0, hasWrapup: false, reviewState: 'OPEN', reviewActive: false,
    claimedAt: null, acceptedAt: null, tagId: null, tagName: null, tagIcon: null, tagColor: null, graphLaidAt: null,
    releaseSkipped: false, kind: 'TASK', epicId: null, quick: false, autoAccepted: false, dependsOn: [], trackerIssueKey: null,
    trackerIssueUrl: null, trackerSyncError: null, trackerContentSynced: true, pullRequestNumber: null, pullRequestUrl: null,
    pullRequestFailure: null, pullRequestMergedAt: null, anchor: 'task:RK-1', updatedAt: '2026-10-01T10:00:00Z'
  }
}

function makeNote(id: string, title: string, contextMode: RekallDocument['contextMode'] = 'FULL'): RekallDocument {
  return {
    id: id as DocumentId, title, kind: 'notes', bodyMarkdown: `# ${title}`, taskId, contextMode,
    anchor: `note:${id}`, updatedAt: '2026-10-01T09:00:00Z'
  }
}

let pinia: Pinia
let router: Router

function seed(notes: RekallDocument[]) {
  const store = useConsoleStore()
  store.tasks = [makeTask()]
  store.selectedTaskId = taskId
  store.documents = notes
  store.isLoading = false
  store.createNote = vi.fn().mockResolvedValue(undefined)
  store.deleteNote = vi.fn().mockResolvedValue(undefined)
  return store
}

async function mountAt(path: string) {
  await router.push(path)
  const wrapper = mount(TaskNotesTab, {
    props: { task: makeTask() },
    attachTo: document.body,
    global: {
      plugins: [pinia, router],
      stubs: {
        NotePane: { template: '<div data-testid="note-pane-stub" />' },
        NoteDeleteConfirm: {
          template: '<div data-testid="confirm"><button data-testid="confirm-yes" @click="$emit(\'confirm\')" /></div>'
        }
      }
    }
  })
  await flushPromises()
  return wrapper
}

describe('TaskNotesTab', () => {
  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)
    router = createRouter({
      history: createMemoryHistory(),
      routes: [{ path: '/task/:id/:tab?/:noteId?', name: 'task', component: stub }]
    })
  })

  afterEach(() => {
    document.body.innerHTML = ''
  })

  it('lists the task’s notes beside the note pane and opens on the first', async () => {
    const store = seed([makeNote('n1', 'first.md'), makeNote('n2', 'second.md')])
    const tab = await mountAt('/task/t1/notes')

    const titles = tab.findAll('[data-testid="task-note"]').map((note) => note.get('.truncate').text())
    expect(titles).toEqual(['first.md', 'second.md'])
    expect(store.selectedDocId).toBe('n1')
    expect(tab.find('[data-testid="note-pane-stub"]').exists()).toBe(true)
  })

  it('puts the note in view in the URL and opens the one a link names', async () => {
    const store = seed([makeNote('n1', 'first.md'), makeNote('n2', 'second.md')])
    const tab = await mountAt('/task/t1/notes/n2')

    expect(store.selectedDocId).toBe('n2')

    await tab.findAll('[data-testid="task-note"]')[0]!.trigger('click')
    await flushPromises()

    expect(router.currentRoute.value.params.noteId).toBe('n1')
  })

  it('marks a note loaded on request with an open ring', async () => {
    seed([makeNote('n1', 'first.md'), makeNote('n2', 'second.md', 'REFERENCE')])
    const tab = await mountAt('/task/t1/notes')

    expect(tab.findAll('[data-testid="task-note-marker"]').map((marker) => marker.attributes('data-load-mode'))).toEqual([
      'in-full',
      'on-request'
    ])
  })

  it('adds information and deletes a note after asking', async () => {
    const store = seed([makeNote('n1', 'first.md')])
    const tab = await mountAt('/task/t1/notes')

    await tab.get('[data-testid="information-add"]').trigger('click')
    expect(store.createNote).toHaveBeenCalledWith(taskId)

    await tab.get('[data-testid="task-note-delete"]').trigger('click')
    await tab.get('[data-testid="confirm-yes"]').trigger('click')
    await flushPromises()

    expect(store.deleteNote).toHaveBeenCalledWith('n1')
  })

  it('invites the first note when there are none', async () => {
    const store = seed([])
    const tab = await mountAt('/task/t1/notes')

    expect(tab.get('[data-testid="notes-empty"]').text()).toContain('Nothing here yet')
    await tab.get('[data-testid="information-add"]').trigger('click')

    expect(store.createNote).toHaveBeenCalledWith(taskId)
  })
})
