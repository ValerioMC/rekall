import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import QueueTaskPicker from '@/features/queue/components/QueueTaskPicker.vue'
import { useConsoleStore } from '@/stores/console.store'
import { taskFixture } from '../../support/taskFixture'
import type { TaskId } from '@/model/branded'

beforeEach(() => setActivePinia(createPinia()))

describe('QueueTaskPicker', () => {
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
