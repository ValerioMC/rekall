import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import QuickTaskMark from '@/features/catalog/components/QuickTaskMark.vue'

function mark(task: { quick: boolean; autoAccepted: boolean; acceptedAt: string | null }, compact = false) {
  return mount(QuickTaskMark, { props: { task, compact } })
}

describe('QuickTaskMark', () => {
  it('says nothing about a task that is not quick', () => {
    expect(mark({ quick: false, autoAccepted: false, acceptedAt: null }).find('[data-testid="quick-task-mark"]').exists()).toBe(false)
  })

  it('labels a quick task that is still waiting for its wrapup', () => {
    const wrapper = mark({ quick: true, autoAccepted: false, acceptedAt: null })
    expect(wrapper.text()).toBe('Quick')
    expect(wrapper.find('[data-testid="auto-accepted-mark"]').exists()).toBe(false)
  })

  it('adds when Rekall accepted it, in words and not by colour alone', () => {
    const wrapper = mark({ quick: true, autoAccepted: true, acceptedAt: new Date().toISOString() })
    expect(wrapper.find('[data-testid="auto-accepted-mark"]').text()).toBe('Auto-accepted just now')
  })

  it('leaves the time out where the row is narrow', () => {
    const wrapper = mark({ quick: true, autoAccepted: true, acceptedAt: new Date().toISOString() }, true)
    expect(wrapper.find('[data-testid="auto-accepted-mark"]').text()).toBe('Auto-accepted')
  })
})
