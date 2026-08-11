import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import CommandPalette from '@/components/shared/CommandPalette.vue'
import { useSchemaStore } from '@/stores/schema.store'
import { asEntityId, asEntityName } from '@/model/branded'
import type { Entity } from '@/model/schema'

const push = vi.fn()
vi.mock('vue-router', () => ({ useRouter: () => ({ push }) }))

function entity(name: string, label: string, applied: boolean): Entity {
  return {
    id: asEntityId(`00000000-0000-0000-0000-00000000000${label.length}`),
    physicalName: asEntityName(name),
    label,
    labelPlural: `${label}s`,
    description: '',
    aliases: [],
    displayFieldId: null,
    status: applied ? 'APPLIED' : 'DRAFT',
    fields: []
  }
}

function mountPalette() {
  const store = useSchemaStore()
  store.entities = [entity('project', 'Project', true), entity('environment', 'Environment', false)]
  return mount(CommandPalette, { attachTo: document.body })
}

function press(key: string, meta = false) {
  window.dispatchEvent(new KeyboardEvent('keydown', { key, metaKey: meta, bubbles: true }))
}

describe('CommandPalette', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    push.mockClear()
  })

  it('stays hidden until the shortcut is pressed', async () => {
    const wrapper = mountPalette()
    expect(wrapper.find('[role=dialog]').exists()).toBe(false)

    press('k', true)
    await wrapper.vm.$nextTick()

    expect(wrapper.find('[role=dialog]').exists()).toBe(true)
  })

  it('lists screens plus every entity, and the data browser only for applied ones', async () => {
    const wrapper = mountPalette()
    press('k', true)
    await wrapper.vm.$nextTick()

    const text = wrapper.text()
    expect(text).toContain('Schema')
    expect(text).toContain('Plan')
    expect(text).toContain('Environment')
    // Environment is a draft, so browsing its records would 404.
    expect(text).toContain('Browse project records')
    expect(text).not.toContain('Browse environment records')
  })

  it('filters on both the label and the physical name', async () => {
    const wrapper = mountPalette()
    press('k', true)
    await wrapper.vm.$nextTick()

    await wrapper.find('input').setValue('environ')

    expect(wrapper.text()).toContain('Environment')
    expect(wrapper.text()).not.toContain('Browse project records')
  })

  it('navigates on enter and closes', async () => {
    const wrapper = mountPalette()
    press('k', true)
    await wrapper.vm.$nextTick()

    press('Enter')
    await wrapper.vm.$nextTick()

    expect(push).toHaveBeenCalledWith('/schema')
    expect(wrapper.find('[role=dialog]').exists()).toBe(false)
  })

  it('closes on escape without navigating', async () => {
    const wrapper = mountPalette()
    press('k', true)
    await wrapper.vm.$nextTick()

    press('Escape')
    await wrapper.vm.$nextTick()

    expect(wrapper.find('[role=dialog]').exists()).toBe(false)
    expect(push).not.toHaveBeenCalled()
  })
})
