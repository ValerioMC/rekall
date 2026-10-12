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

const id = 't1' as TaskId
const tagId = 'g1' as TagId

function taskWith(overrides: Partial<Task> = {}): Task {
  return {
    id,
    label: 'add-skill',
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
  it('says there is no tag, and offers to add a skill', () => {
    const wrapper = mount(TaskLabelsStrip, { props: { task: taskWith() } })

    expect(wrapper.get('[data-testid="task-no-tag"]').text()).toBe('No tag')
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
})
