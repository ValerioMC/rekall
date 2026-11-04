import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import DeviationsCard from '@/features/deviations/components/DeviationsCard.vue'
import { deviation } from './fixtures'

function card(steps: number[], openStep: number | null = null) {
  return mount(DeviationsCard, { props: { deviations: steps.map((step) => deviation(step)), openStep } })
}

describe('the deviations row of the dossier', () => {
  it('is not drawn for a task whose work followed the plan', () => {
    expect(card([]).find('[data-testid="deviations-card"]').exists()).toBe(false)
  })

  it('counts the deviations and gives one link per step, opening that step', async () => {
    const wrapper = card([2, 5])

    expect(wrapper.get('[data-testid="deviations-count"]').text()).toBe('2')
    const links = wrapper.findAll('[data-testid="deviation-link"]')
    expect(links.map((link) => link.text())).toEqual(['Step 2', 'Step 5'])
    expect(links[1]!.attributes('title')).toBe('Step 5: Title of step 5')

    await links[1]!.trigger('click')

    expect(wrapper.emitted('open')).toEqual([[5]])
  })

  it('marks the link of the deviation that is open', () => {
    const links = card([2, 5], 5).findAll('[data-testid="deviation-link"]')

    expect(links.map((link) => link.attributes('aria-current'))).toEqual(['false', 'true'])
  })

  it('keeps a long run to a few links and opens the first one hidden from the count', async () => {
    const wrapper = card([1, 2, 3, 4, 5, 6, 7, 8])

    expect(wrapper.findAll('[data-testid="deviation-link"]')).toHaveLength(6)
    const more = wrapper.get('[data-testid="deviation-more"]')
    expect(more.text()).toBe('+2 more')

    await more.trigger('click')

    expect(wrapper.emitted('open')).toEqual([[7]])
  })
})
