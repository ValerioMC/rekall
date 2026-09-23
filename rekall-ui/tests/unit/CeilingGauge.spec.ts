import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import CeilingGauge from '@/components/queue/CeilingGauge.vue'
import type { ClaudeUsageLimit } from '@/model/claude'

const windows: ClaudeUsageLimit[] = [
  { key: 'session', label: 'Session', percent: 88, severity: 'WARNING', resetsAt: null },
  { key: 'weekly_all', label: 'Weekly · all models', percent: 30, severity: 'NORMAL', resetsAt: null }
]

describe('the ceiling gauge', () => {
  it('marks the windows at or over the line as the ones that will hold the queue', () => {
    const wrapper = mount(CeilingGauge, { props: { modelValue: 85, windows } })

    const fills = wrapper.findAll('[data-reached]')
    expect(fills.map((fill) => fill.attributes('data-reached'))).toEqual(['true', 'false'])
    expect(fills[0]!.classes()).toContain('bg-warn')
    expect(wrapper.find('[data-testid="ceiling-capsule"]').text()).toBe('85%')
  })

  it('moves the line from the slider, and will not let it under the floor', async () => {
    const wrapper = mount(CeilingGauge, { props: { modelValue: 85, windows } })
    const input = wrapper.find('[data-testid="ceiling-input"]')

    await input.setValue('70')
    await input.setValue('3')

    expect(wrapper.emitted('update:modelValue')).toEqual([[70], [10]])
  })

  it('says the line still holds when there is no reading to draw', () => {
    const wrapper = mount(CeilingGauge, { props: { modelValue: 85, windows: [] } })

    expect(wrapper.text()).toContain('No usage reading yet')
  })
})
