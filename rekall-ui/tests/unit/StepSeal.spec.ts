import { afterEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import StepSeal from '@/components/console/StepSeal.vue'

describe('StepSeal', () => {
  afterEach(() => {
    vi.useRealTimers()
  })

  it('draws a pencil in crop marks for a draft, with no ring or check', () => {
    const seal = mount(StepSeal, { props: { state: 'DRAFT' } }).get('[data-testid="step-seal"]')

    expect(seal.attributes('data-state')).toBe('DRAFT')
    expect(seal.find('.seal-pencil').exists()).toBe(true)
    expect(seal.find('.seal-ring').exists()).toBe(false)
    expect(seal.find('.seal-check').exists()).toBe(false)
  })

  it('aims a four-tick reticle at the next open step only', () => {
    const plain = mount(StepSeal, { props: { state: 'OPEN' } })
    const next = mount(StepSeal, { props: { state: 'OPEN', next: true } })

    expect(plain.findAll('.seal-ticks path')).toHaveLength(0)
    expect(next.findAll('.seal-ticks path')).toHaveLength(4)
    expect(next.find('.seal-core').exists()).toBe(true)
    expect(next.get('[data-testid="step-seal"]').attributes('data-next')).toBe('true')
  })

  it('lights the full bezel and orbits a comet while running, with no check', () => {
    const wrapper = mount(StepSeal, { props: { state: 'RUNNING' } })

    expect(wrapper.findAll('.seal-ticks path')).toHaveLength(12)
    expect(wrapper.find('.seal-comet').exists()).toBe(true)
    expect(wrapper.find('.seal-check').exists()).toBe(false)
  })

  it('staggers the ticks so each peaks as the comet head crosses it', () => {
    const delays = mount(StepSeal, { props: { state: 'RUNNING' } })
      .findAll('.seal-ticks path')
      .map((tick) => Number.parseFloat(tick.attributes('style')?.match(/-?[\d.]+s/)?.[0] ?? 'NaN'))

    expect(delays.every((delay) => delay <= 0 && delay > -3.4)).toBe(true)
    expect(new Set(delays).size).toBe(12)
  })

  it('opens the ring of a claimed seal and engraves only a done one', () => {
    const claimed = mount(StepSeal, { props: { state: 'CLAIMED' } })
    const done = mount(StepSeal, { props: { state: 'DONE' } })

    expect(claimed.get('.seal-ring').attributes('transform')).toBe('rotate(25 12 12)')
    expect(claimed.find('.seal-engrave').exists()).toBe(false)
    expect(done.find('.seal-engrave').exists()).toBe(true)
    expect(done.find('.seal-check').exists()).toBe(true)
  })

  it('marks itself complete when the whole checklist is done', () => {
    const seal = mount(StepSeal, { props: { state: 'DONE', complete: true } })

    expect(seal.get('[data-testid="step-seal"]').attributes('data-complete')).toBe('true')
  })

  it('plays an arrival once on a change of state and never on mount', async () => {
    vi.useFakeTimers()
    const wrapper = mount(StepSeal, { props: { state: 'CLAIMED' } })
    const seal = wrapper.get('[data-testid="step-seal"]')

    expect(seal.attributes('data-arrive')).toBeUndefined()

    await wrapper.setProps({ state: 'DONE' })
    expect(seal.attributes('data-arrive')).toBe('DONE')
    expect(wrapper.find('.seal-ripple').exists()).toBe(true)

    vi.advanceTimersByTime(1100)
    await wrapper.vm.$nextTick()
    expect(seal.attributes('data-arrive')).toBeUndefined()
    expect(wrapper.find('.seal-ripple').exists()).toBe(false)
  })
})
