import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import StepPassBars from '@/features/steps/components/StepPassBars.vue'
import { step } from '../adherence/fixtures'

describe('the pass bars of a step', () => {
  it('draw one bar for each pass, in the colour of what became of it', () => {
    const bars = mount(StepPassBars, { props: { step: step(1, 'DONE', 2) } })

    expect(bars.findAll('[data-testid="step-pass-bar"]').map((bar) => bar.attributes('data-kind'))).toEqual([
      'sent-back',
      'sent-back',
      'accepted'
    ])
    expect(bars.get('[data-testid="step-passes"]').attributes('aria-label')).toBe('3 passes, 2 sent back')
  })

  it('mark the pass a session has in hand apart from an accepted one', () => {
    const bars = mount(StepPassBars, { props: { step: step(1, 'CLAIMED') } })

    expect(bars.get('[data-testid="step-pass-bar"]').attributes('data-kind')).toBe('in-hand')
  })

  it('draw nothing for a step nobody started', () => {
    expect(mount(StepPassBars, { props: { step: step(1, 'OPEN') } }).find('[data-testid="step-passes"]').exists()).toBe(false)
  })
})
