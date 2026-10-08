import { mount } from '@vue/test-utils'
import { afterEach, describe, expect, it } from 'vitest'
import { defineComponent, h } from 'vue'
import AppDialog from '@/components/ui/AppDialog.vue'
import { enterDialog, isTopDialog, leaveDialog } from '@/common/a11y/dialog-stack'
import { useModalGate } from '@/composables/useModalGate'

function pressEscape(): KeyboardEvent {
  const event = new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true })
  window.dispatchEvent(event)
  return event
}

const mounted: Array<{ unmount: () => void }> = []

function mountDialog(props: Record<string, unknown> = {}) {
  const wrapper = mount(AppDialog, {
    props: { label: 'Settings', ...props },
    slots: { default: '<button data-testid="first">First</button><button data-testid="last">Last</button>' },
    attachTo: document.body
  })
  mounted.push(wrapper)
  return wrapper
}

afterEach(() => {
  while (mounted.length) mounted.pop()!.unmount()
})

describe('dialog stack', () => {
  it('names only the dialog opened last as the one on top, and hands the top back when it leaves', () => {
    const below = enterDialog()
    const above = enterDialog()

    expect(isTopDialog(above)).toBe(true)
    expect(isTopDialog(below)).toBe(false)

    leaveDialog(above)
    expect(isTopDialog(below)).toBe(true)
    leaveDialog(below)
  })

  it('lets a dialog that is not on top leave without disturbing the order', () => {
    const first = enterDialog()
    const second = enterDialog()
    const third = enterDialog()

    leaveDialog(second)

    expect(isTopDialog(third)).toBe(true)
    leaveDialog(third)
    expect(isTopDialog(first)).toBe(true)
    leaveDialog(first)
  })
})

describe('AppDialog', () => {
  it('draws a modal panel with its label and counts itself as open while mounted', () => {
    const { isModalOpen } = useModalGate()
    const before = isModalOpen.value
    const wrapper = mountDialog({ panelTestid: 'settings-panel' })

    const panel = wrapper.get('[data-testid="settings-panel"]')
    expect(panel.attributes('role')).toBe('dialog')
    expect(panel.attributes('aria-modal')).toBe('true')
    expect(panel.attributes('aria-label')).toBe('Settings')
    expect(isModalOpen.value).toBe(before + 1)

    wrapper.unmount()
    mounted.pop()
    expect(isModalOpen.value).toBe(before)
  })

  it('dismisses on Escape and on the backdrop, saying which', async () => {
    const wrapper = mountDialog()

    pressEscape()
    await wrapper.trigger('click')

    expect(wrapper.emitted('dismiss')).toEqual([['escape'], ['backdrop']])
  })

  it('lets Escape through untouched while it cannot be dismissed, so a control inside can use it', async () => {
    const wrapper = mountDialog({ dismissible: false })

    const event = pressEscape()
    await wrapper.trigger('click')

    expect(wrapper.emitted('dismiss')).toBeUndefined()
    expect(event.cancelBubble).toBe(false)
  })

  it('answers Escape only when it is the dialog on top, so a confirm over a panel closes alone', () => {
    const panel = mountDialog({ label: 'Run queue' })
    const confirm = mountDialog({ label: 'Stop the queue?', role: 'alertdialog' })

    pressEscape()

    expect(confirm.emitted('dismiss')).toEqual([['escape']])
    expect(panel.emitted('dismiss')).toBeUndefined()
  })

  it('keeps Tab inside the panel, wrapping from the last control to the first', () => {
    const wrapper = mountDialog()
    const last = wrapper.get<HTMLButtonElement>('[data-testid="last"]').element
    // happy-dom has no layout, so offsetParent is null; the trap filters on it.
    for (const button of wrapper.findAll('button')) Object.defineProperty(button.element, 'offsetParent', { value: document.body })
    last.focus()

    const tab = new KeyboardEvent('keydown', { key: 'Tab', bubbles: true, cancelable: true })
    window.dispatchEvent(tab)

    expect(tab.defaultPrevented).toBe(true)
    expect(document.activeElement).toBe(wrapper.get('[data-testid="first"]').element)
  })

  it('renders its overlay slot outside the panel', () => {
    const wrapper = mount(
      defineComponent(() => () =>
        h(AppDialog, { label: 'Tags', panelTestid: 'panel' }, { default: () => 'body', overlay: () => h('p', { 'data-testid': 'overlay' }) })
      ),
      { attachTo: document.body }
    )
    mounted.push(wrapper)

    expect(wrapper.get('[data-testid="panel"]').find('[data-testid="overlay"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="overlay"]').exists()).toBe(true)
  })
})
