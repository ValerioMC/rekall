import { beforeEach, describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import ContextFillBadge from '@/features/context-fill/components/ContextFillBadge.vue'
import { useConsoleStore } from '@/stores/console.store'
import { fill, OTHER_STEP, STEP, TASK } from './fixtures'

function mountBadge(stepId: typeof STEP | null, fills = [fill(null, 40), fill(STEP, 90)]) {
  const store = useConsoleStore()
  store.applyContextFillEvent({ taskId: TASK, fills })
  return mount(ContextFillBadge, { props: { subject: { taskId: TASK, stepId } } })
}

describe('ContextFillBadge', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('shows the percent of the task, or of a step, through one prop', () => {
    expect(mountBadge(null).text()).toBe('40%')
    expect(mountBadge(STEP).text()).toBe('90%')
  })

  it('shows nothing when no turn recorded a fill for it', () => {
    expect(mountBadge(OTHER_STEP).find('[data-testid=context-fill-badge]').exists()).toBe(false)
  })

  it('is neutral below 60, warning from 60 and danger from 85', () => {
    const levels = [30, 60, 85].map((percent) =>
      mountBadge(null, [fill(null, percent)]).get('[data-testid=context-fill-badge]').attributes('data-level')
    )

    expect(levels).toEqual(['neutral', 'warning', 'danger'])
  })

  it('fills its bar by the percent', () => {
    const badge = mountBadge(null, [fill(null, 40)])

    expect(badge.get('span span span').attributes('style')).toContain('width: 40%')
  })

  it('says the tokens over the window and when it was recorded in its tooltip', () => {
    const badge = mountBadge(null, [fill(null, 40)])
    const title = badge.get('[data-testid=context-fill-badge]').attributes('title')

    expect(title).toContain('80k of 200k tokens of the context window')
    expect(title).toContain('recorded just now')
  })
})
