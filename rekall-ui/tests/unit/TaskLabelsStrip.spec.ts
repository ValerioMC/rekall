import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import TaskLabelsStrip from '@/components/console/TaskLabelsStrip.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useClaudeResourcesStore } from '@/stores/claudeResources.store'
import { catalog } from './claude-resources.fixtures'
import type { Task } from '@/model/catalog'
import type { TagId, TaskId } from '@/model/branded'

const api = vi.hoisted(() => ({ setTaskSkills: vi.fn(), setStepSkills: vi.fn(), setTagSkills: vi.fn(), fetchSkillAssignments: vi.fn() }))
vi.mock('@/api/skill-assignments.api', () => api)

const catalogApi = vi.hoisted(() => ({ updateTask: vi.fn() }))
vi.mock('@/api/catalog.api', async (original) => ({ ...(await original<object>()), updateTask: catalogApi.updateTask }))

const id = 't1' as TaskId
const tagId = 'g1' as TagId

function taskWith(overrides: Partial<Task> = {}): Task {
  return {
    id,
    label: 'add-skill',
    projectId: 'p1',
    title: 'Add skill',
    status: 'TODO',
    tagId: null,
    tagName: null,
    tagIcon: null,
    tagColor: null,
    ...overrides
  } as Task
}

beforeEach(() => {
  vi.resetAllMocks()
  document.body.innerHTML = ''
  setActivePinia(createPinia())
  const resources = useClaudeResourcesStore()
  resources.catalog = catalog()
  resources.loaded = true
})

describe('TaskLabelsStrip', () => {
  it('offers to add a tag and a skill when the task has neither', () => {
    const wrapper = mount(TaskLabelsStrip, { props: { task: taskWith() } })

    expect(wrapper.get('[data-testid="task-tag-trigger"]').text()).toBe('New tag')
    expect(wrapper.get('[data-testid="skill-add"]').text()).toBe('Add a skill')
  })

  it("shows the task's own skills removable and the tag's dashed", () => {
    const store = useConsoleStore()
    store.taskSkills = [{ taskId: id, stepId: null, skillName: 'rust-standards' }]
    store.tagSkills = [{ tagId, skillName: 'code-review' }]
    const wrapper = mount(TaskLabelsStrip, {
      props: { task: taskWith({ tagId, tagName: 'Backend', tagIcon: 'star', tagColor: 'crimson' }) }
    })

    const chips = wrapper.findAll('[data-testid="skill-chip"]')

    expect(chips.map((chip) => chip.attributes('data-origin'))).toEqual(['tag', 'task'])
    expect(chips[0]!.find('[data-testid="skill-chip-remove"]').exists()).toBe(false)
    expect(wrapper.text()).toContain('Backend')
  })

  it('sends the whole list without the skill that was removed', async () => {
    const store = useConsoleStore()
    store.taskSkills = [
      { taskId: id, stepId: null, skillName: 'a' },
      { taskId: id, stepId: null, skillName: 'b' }
    ]
    api.setTaskSkills.mockResolvedValue([{ taskId: id, stepId: null, skillName: 'b' }])
    const wrapper = mount(TaskLabelsStrip, { props: { task: taskWith() } })

    await wrapper.findAll('[data-testid="skill-chip-remove"]')[0]!.trigger('click')
    await flushPromises()

    expect(api.setTaskSkills).toHaveBeenCalledWith(id, ['b'])
  })

  it('picks a tag from the list and saves it at once', async () => {
    const store = useConsoleStore()
    store.tags = [{ id: tagId, name: 'Backend', icon: 'star', color: 'crimson' }] as never
    store.tasks = [taskWith()]
    catalogApi.updateTask.mockResolvedValue(taskWith({ tagId, tagName: 'Backend', tagIcon: 'star', tagColor: 'crimson' }))
    const wrapper = mount(TaskLabelsStrip, { props: { task: taskWith() }, attachTo: document.body })

    await wrapper.get('[data-testid="task-tag-trigger"]').trigger('click')
    const rows = document.body.querySelectorAll<HTMLElement>('[data-testid="tag-picker-row"]')

    expect([...rows].map((row) => row.textContent?.trim())).toEqual(['No tag', 'Backend'])
    rows[1]!.click()
    await flushPromises()

    expect(catalogApi.updateTask).toHaveBeenCalledWith(id, expect.objectContaining({ tagId }))
    expect(document.body.querySelector('[data-testid="tag-picker"]')).toBeNull()
    wrapper.unmount()
  })

  it('says so when no tag is configured', async () => {
    const wrapper = mount(TaskLabelsStrip, { props: { task: taskWith() }, attachTo: document.body })

    await wrapper.get('[data-testid="task-tag-trigger"]').trigger('click')

    expect(document.body.querySelector('[data-testid="tag-picker-empty"]')).not.toBeNull()
    wrapper.unmount()
  })
})
