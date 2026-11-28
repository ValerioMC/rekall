import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import DropPanel from '@/components/ui/DropPanel.vue'

function render() {
  return mount(DropPanel, {
    props: { label: 'Options', open: true, 'onUpdate:open': (open: boolean) => void wrapper.setProps({ open }) },
    slots: { default: '<input data-testid="inside" />' }
  })
}

let wrapper: ReturnType<typeof render>

describe('DropPanel', () => {
  it('stays open while its controls are used', async () => {
    wrapper = render()

    await wrapper.get('[data-testid="inside"]').trigger('click')

    expect(wrapper.find('[data-testid="drop-panel-body"]').exists()).toBe(true)
  })

  it('closes on Escape', async () => {
    wrapper = render()

    await wrapper.get('[data-testid="inside"]').trigger('keydown', { key: 'Escape' })

    expect(wrapper.find('[data-testid="drop-panel-body"]').exists()).toBe(false)
  })

  it('closes on a press outside', async () => {
    wrapper = render()

    await wrapper.get('button[aria-hidden="true"]').trigger('click')

    expect(wrapper.find('[data-testid="drop-panel-body"]').exists()).toBe(false)
  })
})
