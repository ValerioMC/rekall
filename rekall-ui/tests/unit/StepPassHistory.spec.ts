import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import StepPassHistory from '@/components/console/StepPassHistory.vue'
import type { StepPass } from '@/model/catalog'

const PASSES: StepPass[] = [
  { detailMarkdown: 'Sum by month', sentBackAt: '2026-09-28T10:00:00Z' },
  { detailMarkdown: 'Group by week instead', sentBackAt: '2026-09-29T10:00:00Z' }
]

function render(passes: StepPass[] = PASSES) {
  return mount(StepPassHistory, {
    props: { passes },
    global: { stubs: { AppMarkdownEditor: { props: ['modelValue'], template: '<div class="md">{{ modelValue }}</div>' } } }
  })
}

describe('the sealed passes of a step sent back', () => {
  it('numbers every pass and opens only the latest, the one the feedback answers', () => {
    const wrapper = render()

    const passes = wrapper.findAll('[data-testid="step-pass"]')
    expect(passes.map((pass) => pass.text())).toEqual([
      expect.stringContaining('Pass 1'),
      expect.stringContaining('Pass 2')
    ])
    expect(wrapper.findAll('.md').map((detail) => detail.text())).toEqual(['Group by week instead'])
  })

  it('unfolds an earlier pass on request', async () => {
    const wrapper = render()

    await wrapper.findAll('[data-testid="step-pass"] button')[0]!.trigger('click')

    expect(wrapper.findAll('.md').map((detail) => detail.text())).toEqual([
      'Sum by month',
      'Group by week instead'
    ])
  })

  it('says a pass worked from the title alone when it had no detail', () => {
    const wrapper = render([{ detailMarkdown: null, sentBackAt: '2026-09-28T10:00:00Z' }])

    expect(wrapper.text()).toContain('Worked from the title alone.')
  })
})
