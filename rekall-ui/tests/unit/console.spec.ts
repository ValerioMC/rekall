import { beforeEach, describe, expect, it, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'
import type { Company, Project, RekallDocument, Task } from '@/model/catalog'
import type { CompanyId, DocumentId, ProjectId, TaskId } from '@/model/branded'

/**
 * The console's reading rules, which is where the screen's behaviour actually lives: what the
 * project scope hides, what the search finds, and what a note being on several tasks means for
 * both. Mounting the panes would test Vue; this tests the decisions.
 */
const acme = 'c1' as CompanyId
const globex = 'c2' as CompanyId

const companies: Company[] = [
  { id: acme, name: 'acme', description: null, projectCount: 1, taskCount: 2, updatedAt: '2026-08-12T10:00:00Z' },
  { id: globex, name: 'globex', description: null, projectCount: 1, taskCount: 1, updatedAt: '2026-08-12T10:00:00Z' }
]

const vega = 'p1' as ProjectId
const beacon = 'p2' as ProjectId

const projects: Project[] = [
  { id: vega, label: 'vega', title: 'Vega Platform', status: 'ACTIVE', description: null, companyId: acme, companyName: 'acme', taskCount: 2, anchor: 'project:vega', updatedAt: '2026-08-12T10:00:00Z' },
  { id: beacon, label: 'beacon', title: 'Beacon', status: 'ACTIVE', description: null, companyId: globex, companyName: 'globex', taskCount: 1, anchor: 'project:beacon', updatedAt: '2026-08-12T10:00:00Z' }
]

const validator = 't1' as TaskId
const retry = 't2' as TaskId
const wiring = 't3' as TaskId

const task = (
  id: TaskId,
  label: string,
  title: string,
  status: Task['status'],
  projectId: ProjectId,
  projectLabel: string,
  projectTitle: string,
  companyName: string
): Task => ({
  id,
  label,
  title,
  status,
  description: null,
  projectId,
  projectLabel,
  projectTitle,
  companyName,
  documentCount: 1,
  anchor: `project:${projectLabel} task:${label}`,
  updatedAt: '2026-08-12T10:00:00Z'
})

const tasks: Task[] = [
  task(validator, 'report-builder', 'Report builder', 'IN_PROGRESS', vega, 'vega', 'Vega Platform', 'acme'),
  task(retry, 'retry-policy', 'Retry policy', 'TODO', vega, 'vega', 'Vega Platform', 'acme'),
  task(wiring, 'wiring', 'Wiring the adapter', 'DONE', beacon, 'beacon', 'Beacon', 'globex')
]

const ref = (id: TaskId, label: string, title: string, projectLabel: string, companyName = 'acme') => ({
  id,
  label,
  title,
  projectLabel,
  projectTitle: projectLabel === 'vega' ? 'Vega Platform' : 'Beacon',
  companyName,
  anchor: `project:${projectLabel} task:${label}`
})

const documents: RekallDocument[] = [
  {
    id: 'd1' as DocumentId,
    title: 'CONTEXT.md',
    kind: 'context',
    bodyMarkdown: 'Il workflow parte da POST /api/v1/pipelines',
    tasks: [ref(validator, 'report-builder', 'Report builder', 'vega')],
    updatedAt: '2026-08-12T12:00:00Z'
  },
  {
    // The shape the whole model change exists for: one note, three tasks.
    id: 'd2' as DocumentId,
    title: 'kmaster14.md',
    kind: 'notes',
    bodyMarkdown: 'Accesso via bastion',
    tasks: [
      ref(validator, 'report-builder', 'Report builder', 'vega'),
      ref(retry, 'retry-policy', 'Retry policy', 'vega')
    ],
    updatedAt: '2026-08-12T13:00:00Z'
  },
  {
    id: 'd3' as DocumentId,
    title: 'onboarding.md',
    kind: 'notes',
    bodyMarkdown: 'brew install openjdk',
    tasks: [ref(wiring, 'wiring', 'Wiring the adapter', 'beacon', 'globex')],
    updatedAt: '2026-08-12T09:00:00Z'
  }
]

const updateTask = vi.fn(async () => ({ ...tasks[0]!, status: 'DONE' as const }))

vi.mock('@/api/catalog.api', () => ({
  fetchCompanies: vi.fn(async () => companies),
  fetchProjects: vi.fn(async () => projects),
  createCompany: vi.fn(),
  updateCompany: vi.fn(),
  deleteCompany: vi.fn(),
  fetchTasks: vi.fn(async () => tasks),
  createProject: vi.fn(),
  updateProject: vi.fn(),
  deleteProject: vi.fn(),
  createTask: vi.fn(),
  updateTask: (...args: unknown[]) => updateTask(...(args as [])),
  deleteTask: vi.fn()
}))

vi.mock('@/api/documents.api', () => ({
  fetchAllDocuments: vi.fn(async () => documents),
  createDocument: vi.fn(),
  updateDocument: vi.fn(),
  deleteDocument: vi.fn()
}))

describe('console store', () => {
  let store: ReturnType<typeof useConsoleStore>

  beforeEach(async () => {
    updateTask.mockClear()
    setActivePinia(createPinia())
    store = useConsoleStore()
    await store.load()
  })

  it('narrows from everything, to a company, to one of its projects', () => {
    expect(store.visibleTasks).toHaveLength(3)

    store.setScope(acme)
    expect(store.visibleTasks.map((t) => t.label)).toEqual(['report-builder', 'retry-policy'])

    store.setScope(acme, vega)
    expect(store.visibleTasks).toHaveLength(2)
    expect(store.scopeName).toBe('acme / Vega Platform')
    expect(store.scopeAnchor).toBe('project:vega')

    store.setScope(null)
    expect(store.visibleTasks).toHaveLength(3)
    expect(store.scopeName).toBe('All work')
  })

  it('finds a task by either half of its anchor', () => {
    store.filter = 'vega/retry'
    expect(store.visibleTasks.map((t) => t.label)).toEqual(['retry-policy'])

    store.filter = 'task:report-builder'
    expect(store.visibleTasks.map((t) => t.label)).toEqual(['report-builder'])
  })

  /**
   * The label is what you type after `/rk` and the title is what you called it out loud. Both
   * have to find the row, or one of them is a name you cannot search by.
   */
  it('finds a task by its title as well as by its label', () => {
    store.filter = 'Wiring the adapter'
    expect(store.visibleTasks.map((t) => t.label)).toEqual(['wiring'])

    store.filter = 'wiring'
    expect(store.visibleTasks.map((t) => t.label)).toEqual(['wiring'])
  })

  it('searches note bodies, not only their titles', () => {
    store.navMode = 'notes'
    store.filter = 'bastion'
    expect(store.visibleDocuments.map((document) => document.title)).toEqual(['kmaster14.md'])
  })

  /**
   * The case that made this necessary: scoped to one project, searching for something that
   * lives in another. Without the notice the answer looks like "you never wrote it".
   */
  it('reports the matches the project scope is hiding', () => {
    store.setScope(acme)
    store.filter = 'wiring'

    expect(store.visibleTasks).toHaveLength(0)
    expect(store.elsewhere).toEqual({ count: 1, names: ['globex'] })

    store.setScope(null)
    expect(store.visibleTasks).toHaveLength(1)
    expect(store.elsewhere).toBeNull()
  })

  it('says nothing about elsewhere when the search is already global', () => {
    store.filter = 'wiring'
    expect(store.elsewhere).toBeNull()
  })

  it('lists a shared note under every task it is attached to', () => {
    store.selectTask(validator)
    expect(store.taskDocuments.map((document) => document.title)).toEqual([
      'CONTEXT.md',
      'kmaster14.md'
    ])

    store.selectTask(retry)
    expect(store.taskDocuments.map((document) => document.title)).toEqual(['kmaster14.md'])
  })

  it('opens the first note of a task when the task is picked', () => {
    store.selectTask(retry)
    expect(store.selectedDocId).toBe('d2')
  })

  /** Picking a note from the Notes list has to bring its task along, or the panes disagree. */
  it('follows a note back to a task it belongs to', () => {
    store.selectTask(validator)
    store.selectDocument('d3' as DocumentId)

    expect(store.selectedTaskId).toBe(wiring)
  })

  it('offers the most recently written notes first', () => {
    expect(store.recentDocuments.map((document) => document.title)).toEqual([
      'kmaster14.md',
      'CONTEXT.md',
      'onboarding.md'
    ])
  })

  it('keeps the selection when the new scope still contains it', () => {
    store.selectTask(validator)
    store.setScope(acme)
    expect(store.selectedTaskId).toBe(validator)
  })

  it('moves the selection into the new scope when the old one falls outside it', () => {
    store.selectTask(wiring)
    store.setScope(acme)
    expect(store.selectedTaskId).toBe(validator)
  })

  /**
   * A status change sends the record back whole. Dropping the label out of that payload would
   * blank the column the anchor resolves on, which the endpoint would then reject or, worse,
   * accept.
   */
  it('keeps the label and the title when only the status changes', async () => {
    await store.setTaskStatus(validator, 'DONE')

    expect(updateTask).toHaveBeenCalledWith(
      validator,
      expect.objectContaining({
        label: 'report-builder',
        title: 'Report builder',
        status: 'DONE',
        projectId: vega
      })
    )
  })
})
