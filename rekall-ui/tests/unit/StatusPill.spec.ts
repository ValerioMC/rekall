import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import StatusPill, { type StatusPillTone } from '@/components/ui/StatusPill.vue'

const TONES: readonly StatusPillTone[] = ['neutral', 'progress', 'review', 'done', 'filed', 'running', 'danger']

describe('StatusPill', () => {
  it.each(TONES)('renders the %s tone with its word', (tone) => {
    const wrapper = mount(StatusPill, { props: { tone }, slots: { default: 'Word' } })
    expect(wrapper.attributes('data-tone')).toBe(tone)
    expect(wrapper.text()).toBe('Word')
  })

  it('is neutral by default and shows the orb only while live', () => {
    const quiet = mount(StatusPill, { slots: { default: 'To do' } })
    expect(quiet.attributes('data-tone')).toBe('neutral')
    expect(quiet.find('[data-testid="status-pill-orb"]').exists()).toBe(false)

    const live = mount(StatusPill, { props: { tone: 'running', live: true }, slots: { default: 'Working' } })
    expect(live.find('[data-testid="status-pill-orb"]').exists()).toBe(true)
  })
})
