import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import TaskActivityMark from '@/features/workingset/components/TaskActivityMark.vue'
import { TASK_ACTIVITIES, TASK_ACTIVITY_LABEL } from '@/features/workingset/model/task-activity'

describe('the task activity mark', () => {
  it.each(TASK_ACTIVITIES)('names %s in its state marker and its label', (activity) => {
    const mark = mount(TaskActivityMark, { props: { activity } })

    expect(mark.attributes('data-activity')).toBe(activity)
    expect(mark.attributes('role')).toBe('img')
    expect(mark.attributes('aria-label')).toBe(TASK_ACTIVITY_LABEL[activity])
    expect(mark.findAll('.activity-bar')).toHaveLength(3)
  })

  it('carries the animated class only while working', () => {
    const animated = TASK_ACTIVITIES.filter(
      (activity) => mount(TaskActivityMark, { props: { activity } }).classes('activity-mark-working')
    )

    expect(animated).toEqual(['WORKING'])
  })

  it('gives each state its own bar colour', () => {
    const colours = TASK_ACTIVITIES.map((activity) => mount(TaskActivityMark, { props: { activity } }).get('.activity-bar').classes().find((name) => name.startsWith('bg-')))

    expect(new Set(colours).size).toBe(3)
  })
})
