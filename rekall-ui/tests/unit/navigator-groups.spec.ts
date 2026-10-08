import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { nextTick, ref } from 'vue'
import { groupTasksByProject } from '@/common/catalog/project-task-groups'
import { useNavigatorShelves } from '@/composables/useNavigatorShelves'
import { taskFixture } from './support/taskFixture'
import type { Project, Task, TaskStatus } from '@/model/catalog'
import type { CompanyId, ProjectId, TaskId } from '@/model/branded'

function task(id: string, projectId: string, status: TaskStatus = 'TODO', projectTitle = projectId, companyName = 'acme'): Task {
  return { ...taskFixture(id as TaskId), projectId: projectId as ProjectId, projectTitle, companyName, status }
}

function project(id: string, over: Partial<Project> = {}): Project {
  return {
    id: id as ProjectId,
    label: id,
    title: id,
    status: 'ACTIVE',
    icon: 'rocket',
    iconImageVersion: null,
    description: null,
    blueprintMarkdown: null,
    repoFolder: null,
    autoCommit: false,
    companyId: 'c1' as CompanyId,
    companyName: 'acme',
    taskCount: 0,
    anchor: `project:${id}`,
    updatedAt: '2026-10-01T00:00:00Z',
    ...over
  }
}

describe('tasks grouped by project', () => {
  it('gathers each project’s tasks into live, backlog and filed, keeping their order', () => {
    const groups = groupTasksByProject(
      [task('a', 'vega'), task('b', 'vega', 'DONE'), task('c', 'vega', 'BACKLOG'), task('d', 'vega', 'IN_PROGRESS')],
      [project('vega')],
      false
    )

    expect(groups).toHaveLength(1)
    expect(groups[0]!.active.map((entry) => entry.id)).toEqual(['a', 'd'])
    expect(groups[0]!.backlog.map((entry) => entry.id)).toEqual(['c'])
    expect(groups[0]!.filed.map((entry) => entry.id)).toEqual(['b'])
    expect(groups[0]!.tasks).toHaveLength(4)
    expect(groups[0]!.icon).toBe('rocket')
  })

  it('puts live projects ahead of paused ones, then orders by title', () => {
    const groups = groupTasksByProject(
      [task('a', 'zeta', 'TODO', 'Zeta'), task('b', 'atlas', 'TODO', 'Atlas'), task('c', 'mira', 'TODO', 'Mira')],
      [project('zeta'), project('atlas', { status: 'PAUSED' }), project('mira')],
      false
    )

    expect(groups.map((group) => group.projectTitle)).toEqual(['Mira', 'Zeta', 'Atlas'])
  })

  it('still draws a card for a project the catalog no longer lists, as active with the default icon', () => {
    const [group] = groupTasksByProject([task('a', 'gone')], [], false)

    expect(group!.status).toBe('ACTIVE')
    expect(group!.icon).toBe('folder')
  })
})

describe('the navigator shelves', () => {
  // This environment hands out no usable storage, so each test gets one of its own.
  beforeEach(() => {
    const values = new Map<string, string>()
    Object.defineProperty(window, 'localStorage', {
      configurable: true,
      value: {
        getItem: (key: string) => values.get(key) ?? null,
        setItem: (key: string, value: string) => void values.set(key, value),
        removeItem: (key: string) => void values.delete(key),
        clear: () => values.clear()
      }
    })
  })
  afterEach(() => delete (window as { localStorage?: unknown }).localStorage)

  it('keeps folded projects across restarts', () => {
    const first = useNavigatorShelves(ref(null))
    first.toggleCollapsed('vega' as ProjectId)

    const second = useNavigatorShelves(ref(null))

    expect(second.collapsedProjects.has('vega' as ProjectId)).toBe(true)
  })

  it('starts unfolded when what was stored cannot be read', () => {
    localStorage.setItem('rekall.nav.collapsed-projects', '{not json')

    expect(useNavigatorShelves(ref(null)).collapsedProjects.size).toBe(0)
  })

  it('opens the shelf of a filed or parked task picked from elsewhere, and leaves it open', async () => {
    const selected = ref<Task | null>(null)
    const shelves = useNavigatorShelves(selected)

    selected.value = task('a', 'vega', 'DONE')
    await nextTick()
    expect(shelves.filedOpen).toBe(true)
    expect(shelves.openFiled.has('vega' as ProjectId)).toBe(true)

    selected.value = task('b', 'vega', 'BACKLOG')
    await nextTick()
    expect(shelves.backlogOpen).toBe(true)
    expect(shelves.openBacklogs.has('vega' as ProjectId)).toBe(true)
    expect(shelves.openFiled.has('vega' as ProjectId)).toBe(true)
  })

  it('toggles the list-wide shelves on their own, without touching a project’s', () => {
    const shelves = useNavigatorShelves(ref(null))

    shelves.toggleListFiled()
    shelves.toggleListBacklog()

    expect(shelves.filedOpen).toBe(true)
    expect(shelves.backlogOpen).toBe(true)
    expect(shelves.openFiled.size).toBe(0)
  })
})
