import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import NavigatorTaskRow from '@/components/console/NavigatorTaskRow.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { Task } from '@/features/catalog/model/catalog'
import type { TagId, TaskId } from '@/model/branded'

/** The mark on a task's row in the navigator: its own skills and its tag's, counted. */
const id = 't1' as TaskId
const tagId = 'g1' as TagId

const task = {
  id,
  label: 'add-skill',
  title: 'Add skill',
  status: 'TODO',
  projectId: 'p1',
  stepCount: 0,
  stepsDone: 0,
  draftStepCount: 0,
  documentCount: 0,
  hasWrapup: false,
  reviewActive: false,
  reviewState: 'OPEN',
  tagId,
  tagName: null,
  tagIcon: null,
  tagColor: null
} as unknown as Task

function mountRow() {
  return mount(NavigatorTaskRow, {
    props: { task, selected: false, running: false, showContext: false, showCompany: false }
  })
}

beforeEach(() => setActivePinia(createPinia()))

describe('NavigatorTaskRow skills', () => {
  it('shows no mark for a task with no skills', () => {
    expect(mountRow().find('[data-testid="skill-mark"]').exists()).toBe(false)
  })

  it('counts the skills of the task and of its tag', () => {
    const store = useConsoleStore()
    store.taskSkills = [{ taskId: id, stepId: null, skillName: 'rust-standards' }]
    store.tagSkills = [{ tagId, skillName: 'code-review' }]

    expect(mountRow().get('[data-testid="skill-mark"]').text()).toBe('2')
  })

  it('does not count a skill that belongs to one of the steps', () => {
    const store = useConsoleStore()
    store.taskSkills = [{ taskId: id, stepId: 's1' as never, skillName: 'ui-ux-pro-max' }]

    expect(mountRow().find('[data-testid="skill-mark"]').exists()).toBe(false)
  })
})
