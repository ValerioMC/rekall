import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import PlanTube from '@/components/ui/PlanTube.vue'

const part = (wrapper: ReturnType<typeof mount>, name: string) => wrapper.find(`[data-testid="plan-tube-${name}"]`)

describe('PlanTube', () => {
  it('fills done, claimed and the running step in proportion', () => {
    const wrapper = mount(PlanTube, { props: { done: 2, claimed: 1, total: 5, live: true } })
    expect(part(wrapper, 'done').attributes('style')).toContain('width: 40%')
    expect(part(wrapper, 'claimed').attributes('style')).toContain('width: 20%')
    expect(part(wrapper, 'running').attributes('style')).toContain('width: 20%')
  })

  it('shows no running sheen unless a session works', () => {
    const wrapper = mount(PlanTube, { props: { done: 1, total: 4 } })
    expect(part(wrapper, 'running').exists()).toBe(false)
    expect(part(wrapper, 'claimed').exists()).toBe(false)
  })

  it('turns green once every step is done', () => {
    expect(part(mount(PlanTube, { props: { done: 3, total: 3 } }), 'done').classes()).toContain('tube-safe')
    expect(part(mount(PlanTube, { props: { done: 2, total: 3 } }), 'done').classes()).not.toContain('tube-safe')
  })

  it('reads out its numbers', () => {
    const wrapper = mount(PlanTube, { props: { done: 2, claimed: 1, total: 5 } })
    expect(wrapper.attributes('aria-label')).toBe('2 of 5 steps done, 1 waiting for review')
  })

  it.each([['sm', 'h-1.5'], ['md', 'h-2'], ['lg', 'h-2.5']] as const)('has the %s height', (size, height) => {
    expect(mount(PlanTube, { props: { done: 0, total: 1, size } }).classes()).toContain(height)
  })
})
