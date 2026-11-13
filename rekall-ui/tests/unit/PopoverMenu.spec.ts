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
})
