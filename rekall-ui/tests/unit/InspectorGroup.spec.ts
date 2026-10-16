import { beforeEach, describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import InspectorGroup from '@/features/project/components/InspectorGroup.vue'

function installStorage(): void {
  const values = new Map<string, string>()
  Object.defineProperty(window, 'localStorage', {
    configurable: true,
    value: {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => void values.set(key, value)
    }
  })
}

const render = (props: Record<string, unknown> = {}) =>
  mount(InspectorGroup, {
    props: { group: 'about', title: 'About', summary: 'A short line', ...props },
    slots: { default: '<p data-testid="body">Body</p>', actions: '<button data-testid="action">+</button>' }
  })

beforeEach(installStorage)

describe('an inspector group', () => {
  it('opens on its body with its heading, count and actions', () => {
    const group = render({ count: 3 })

    expect(group.get('#about-heading').text()).toBe('About')
    expect(group.get('[data-testid="about-count"]').text()).toBe('3')
    expect(group.find('[data-testid="action"]').exists()).toBe(true)
    expect(group.get('#about-body').attributes('style') ?? '').not.toContain('display: none')
    expect(group.find('[data-testid="about-summary"]').exists()).toBe(false)
  })

  it('shows no count when there is nothing to count', () => {
    expect(render({ count: 0 }).find('[data-testid="about-count"]').exists()).toBe(false)
  })

  it('folds to its summary, and a new mount remembers the fold', async () => {
    const group = render()

    await group.get('[data-testid="about-toggle"]').trigger('click')

    expect(group.get('[data-testid="about-toggle"]').attributes('aria-expanded')).toBe('false')
    expect(group.get('#about-body').attributes('style')).toContain('display: none')
    expect(group.get('[data-testid="about-summary"]').text()).toBe('A short line')
    expect(render().get('#about-body').attributes('style')).toContain('display: none')
  })

  it('opens a folded body when asked to expand, and leaves an open one open', async () => {
    const group = render()
    await group.get('[data-testid="about-toggle"]').trigger('click')

    ;(group.vm as unknown as { expand: () => void }).expand()
    await group.vm.$nextTick()
    expect(group.get('#about-body').attributes('style') ?? '').not.toContain('display: none')

    ;(group.vm as unknown as { expand: () => void }).expand()
    await group.vm.$nextTick()
    expect(group.get('#about-body').attributes('style') ?? '').not.toContain('display: none')
  })
})
