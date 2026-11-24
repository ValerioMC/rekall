import { beforeEach, describe, expect, it } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import PopoverMenu, { type PopoverMenuEntry } from '@/components/ui/PopoverMenu.vue'

const ENTRIES: readonly PopoverMenuEntry[] = [
  { kind: 'heading', label: 'Status' },
  { kind: 'item', id: 'todo', label: 'To do' },
  { kind: 'item', id: 'done', label: 'Done', current: true },
  { kind: 'separator' }
]

function mountMenu(open: boolean) {
  return mount(PopoverMenu, {
    props: { entries: ENTRIES, label: 'Status', open },
    slots: { trigger: '<button data-popover-trigger data-testid="trigger">Status</button>' },
    attachTo: document.body
  })
}

describe('PopoverMenu', () => {
  beforeEach(() => {
    document.body.innerHTML = ''
  })

  it('draws nothing until it is open', () => {
    expect(mountMenu(false).find('[data-testid="popover-menu-list"]').exists()).toBe(false)
  })

  it('lists headings, items and separators and marks the current item', async () => {
    const wrapper = mountMenu(true)
    await flushPromises()
    expect(wrapper.findAll('[data-testid="popover-menu-item"]').map((item) => item.text())).toEqual(['To do', 'Done'])
    expect(wrapper.text()).toContain('Status')
    expect(wrapper.find('[role="separator"]').exists()).toBe(true)
    expect(wrapper.get('[aria-current="true"]').text()).toBe('Done')
    expect(document.activeElement?.textContent).toContain('Done')
  })

  it('emits the picked item and closes', async () => {
    const wrapper = mountMenu(true)
    await flushPromises()
    await wrapper.findAll('[data-testid="popover-menu-item"]')[0]!.trigger('click')
    expect(wrapper.emitted('pick')).toEqual([['todo']])
    expect(wrapper.emitted('update:open')?.at(-1)).toEqual([false])
  })

  it('closes on Escape and gives the focus back to its trigger', async () => {
    const wrapper = mountMenu(true)
    await flushPromises()
    await wrapper.get('[data-testid="popover-menu-list"]').trigger('keydown', { key: 'Escape' })
    expect(wrapper.emitted('update:open')?.at(-1)).toEqual([false])
    expect(document.activeElement).toBe(wrapper.get('[data-testid="trigger"]').element)
  })

  it('walks the items with the arrow keys', async () => {
    const wrapper = mountMenu(true)
    await flushPromises()
    await wrapper.get('[data-testid="popover-menu-list"]').trigger('keydown', { key: 'ArrowDown' })
    expect(document.activeElement?.textContent).toContain('To do')
  })

  it('keeps a disabled item in view with its shortcut, passes over it with the arrow keys and never emits it', async () => {
    const wrapper = mount(PopoverMenu, {
      props: {
        entries: [
          { kind: 'item', id: 'up', label: 'Move up', shortcut: '⌥ ↑', disabled: true, hint: 'Nothing open above it' },
          { kind: 'item', id: 'down', label: 'Move down', shortcut: '⌥ ↓' }
        ],
        label: 'Moves',
        open: false
      },
      slots: { trigger: '<button data-popover-trigger>Moves</button>' },
      attachTo: document.body
    })
    await wrapper.setProps({ open: true })
    await flushPromises()
    const [up, down] = wrapper.findAll('[data-testid="popover-menu-item"]')

    expect(up!.attributes('disabled')).toBeDefined()
    expect(up!.text()).toContain('⌥ ↑')
    expect(up!.text()).toContain('Nothing open above it')
    expect(document.activeElement).toBe(down!.element)

    await up!.trigger('click')
    expect(wrapper.emitted('pick')).toBeUndefined()
  })
})
