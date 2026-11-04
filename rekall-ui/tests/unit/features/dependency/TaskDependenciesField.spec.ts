import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import TaskDependenciesField from '@/features/dependency/components/TaskDependenciesField.vue'
import * as api from '@/features/dependency/api/dependencies.api'
import { useConsoleStore } from '@/stores/console.store'
import { taskFixture } from '../../support/taskFixture'
import type { TaskDependency } from '@/features/dependency/model/dependency'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskId } from '@/model/branded'

vi.mock('@/features/dependency/api/dependencies.api')

const report = 'report' as TaskId
const exportTask = 'export' as TaskId
const archive = 'archive' as TaskId
const release = 'release' as TaskId

function dependency(taskId: TaskId, title: string, met: boolean, awaitingReview = false): TaskDependency {
  return { taskId, anchor: `project:vega task:${taskId}`, title, met, awaitingReview }
}

function withTitle(id: TaskId, title: string, overrides: Partial<Task> = {}): Task {
  return { ...taskFixture(id), title, ...overrides }
}

async function render(task: Task = withTitle(exportTask, 'Export')) {
  useConsoleStore().tasks = [
    withTitle(report, 'Report'),
    task,
    withTitle(archive, 'Archive'),
    withTitle(release, 'Release', { kind: 'EPIC' }),
    { ...withTitle('site' as TaskId, 'Banner'), projectId: 'p2' as Task['projectId'] }
  ]
  const wrapper = mount(TaskDependenciesField, { props: { task }, attachTo: document.body })
  await flushPromises()
  return wrapper
}

const offeredTitles = (): string[] =>
  Array.from(document.body.querySelectorAll('[data-testid="task-dependency-picker-row"]')).map(
    (row) => row.querySelector('span:nth-of-type(2)')?.textContent ?? ''
  )

describe('the Depends on field of a task', () => {
  beforeEach(() => {
    document.body.innerHTML = ''
    setActivePinia(createPinia())
    vi.resetAllMocks()
    vi.mocked(api.fetchDependencies).mockResolvedValue([])
  })

  it('offers only the other plain tasks of the same project', async () => {
    const wrapper = await render()

    await wrapper.get('[data-testid="task-dependency-add"]').trigger('click')

    expect(offeredTitles()).toEqual(['Archive', 'Report'])
  })

  it('says where each dependency stands, in words and not only in colour', async () => {
    vi.mocked(api.fetchDependencies).mockResolvedValue([
      dependency(report, 'Report', true),
      dependency(archive, 'Archive', false, true)
    ])

    const wrapper = await render()

    expect(wrapper.findAll('[data-testid="task-dependency"]').map((chip) => chip.text())).toEqual([
      'ReportAccepted',
      'ArchiveClaimed, waiting for your review'
    ])
    await wrapper.get('[data-testid="task-dependency-add"]').trigger('click')

    expect(offeredTitles()).toEqual([])
  })

  it('adds the picked task and shows the list the server answers with', async () => {
    vi.mocked(api.addDependency).mockResolvedValue([dependency(report, 'Report', false)])
    const wrapper = await render()

    await wrapper.get('[data-testid="task-dependency-add"]').trigger('click')
    document.body.querySelectorAll<HTMLButtonElement>('[data-testid="task-dependency-picker-row"]')[1]?.click()
    await flushPromises()

    expect(api.addDependency).toHaveBeenCalledWith(exportTask, report)
    expect(wrapper.findAll('[data-testid="task-dependency"]')).toHaveLength(1)
    expect(document.body.querySelector('[data-testid="task-dependency-picker"]')).toBeNull()
  })

  it('takes a dependency off', async () => {
    vi.mocked(api.fetchDependencies).mockResolvedValue([dependency(report, 'Report', false)])
    vi.mocked(api.removeDependency).mockResolvedValue([])
    const wrapper = await render()

    await wrapper.get('[data-testid="task-dependency-remove"]').trigger('click')
    await flushPromises()

    expect(api.removeDependency).toHaveBeenCalledWith(exportTask, report)
    expect(wrapper.findAll('[data-testid="task-dependency"]')).toHaveLength(0)
  })

  it('is not drawn for an epic', async () => {
    const wrapper = await render(withTitle(release, 'Release', { kind: 'EPIC' }))

    expect(wrapper.find('[data-testid="task-dependencies"]').exists()).toBe(false)
  })
})
