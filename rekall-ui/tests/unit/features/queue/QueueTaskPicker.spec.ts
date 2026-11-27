import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import QueueTaskPicker from '@/features/queue/components/QueueTaskPicker.vue'
import { useConsoleStore } from '@/stores/console.store'
import { taskFixture } from '../../support/taskFixture'
import type { TaskId } from '@/model/branded'

beforeEach(() => setActivePinia(createPinia()))

describe('QueueTaskPicker', () => {
  it('selects a matching task by keyboard and leaves focus on the picker', async () => {
    const task = taskFixture('checkout-pay' as TaskId)
    useConsoleStore().tasks = [task]
    const wrapper = mount(QueueTaskPicker, { props: { excluded: new Set<TaskId>() }, attachTo: document.body })
    const input = wrapper.get('input')

    await input.setValue('checkout')
    await input.trigger('keydown', { key: 'Enter' })

    expect(wrapper.emitted('pick')).toEqual([[task]])
    expect(input.element.value).toBe('')
    expect(document.activeElement).toBe(input.element)
    wrapper.unmount()
  })

  it('never offers an epic to run', async () => {
    useConsoleStore().tasks = [
      { ...taskFixture('checkout-epic' as TaskId), kind: 'EPIC' },
      taskFixture('checkout-pay' as TaskId)
    ]
    const wrapper = mount(QueueTaskPicker, { props: { excluded: new Set<TaskId>() } })

    await wrapper.get('input').setValue('checkout')

    expect(wrapper.text()).toContain('checkout-pay')
    expect(wrapper.text()).not.toContain('checkout-epic')
  })
})
