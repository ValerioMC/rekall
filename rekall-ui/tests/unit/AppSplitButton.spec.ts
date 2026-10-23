import { mount } from '@vue/test-utils'
import { afterEach, describe, expect, it } from 'vitest'
import AppSplitButton from '@/components/ui/AppSplitButton.vue'

const OPTIONS = [
  { id: 'narrow', label: 'Only some', hint: 'Leaves the rest out' },
  { id: 'other', label: 'Something else', hint: 'Another way' }
] as const

function splitButton(options: readonly { id: string; label: string; hint: string }[] = OPTIONS) {
  return mount(AppSplitButton, { props: { options, menuLabel: 'Other ways' }, slots: { default: 'Do it all' }, attachTo: document.body })
}

function menu(): HTMLElement | null {
  return document.body.querySelector<HTMLElement>('[data-testid="split-button-menu"]')
}

describe('AppSplitButton', () => {
  afterEach(() => {
    document.body.innerHTML = ''
  })

  it('runs its main action from the left part of the key', async () => {
    const view = splitButton()
    await view.get('button').trigger('click')
    expect(view.emitted('click')).toHaveLength(1)
    expect(menu()).toBeNull()
  })

  it('opens the alternatives from the arrow and emits the one chosen, then closes', async () => {
    const view = splitButton()
    const arrow = view.get('[data-testid="split-button-arrow"]')
    await arrow.trigger('click')
    expect(arrow.attributes('aria-expanded')).toBe('true')
    expect(menu()?.querySelectorAll('[role="menuitem"]')).toHaveLength(2)
    document.body.querySelector<HTMLElement>('[data-testid="split-button-option-other"]')?.click()
    await view.vm.$nextTick()
    expect(view.emitted('choose')).toEqual([['other']])
    expect(menu()).toBeNull()
  })

  it('closes the menu on Escape without choosing', async () => {
    const view = splitButton()
    await view.get('[data-testid="split-button-arrow"]').trigger('click')
    menu()?.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    await view.vm.$nextTick()
    expect(menu()).toBeNull()
    expect(view.emitted('choose')).toBeUndefined()
  })

  it('is a plain button when there is no alternative', () => {
    const view = splitButton([])
    expect(view.find('[data-testid="split-button-arrow"]').exists()).toBe(false)
  })

  it('keeps the menu shut while the main action is running', () => {
    const view = mount(AppSplitButton, { props: { options: OPTIONS, menuLabel: 'Other ways', loading: true } })
    expect(view.get('[data-testid="split-button-arrow"]').attributes('disabled')).toBeDefined()
  })
})
