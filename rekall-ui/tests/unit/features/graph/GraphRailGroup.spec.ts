import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import GraphRailGroup from '@/features/graph/components/GraphRailGroup.vue'
import { aGraph, PROJECT } from './graph.fixtures'

function group(props: Partial<InstanceType<typeof GraphRailGroup>['$props']> = {}) {
  return mount(GraphRailGroup, {
    props: { projectId: PROJECT, graphs: [], selectedId: null, settingUp: false, loading: false, ...props }
  })
}

// This environment hands out no usable storage, so each test gets one of its own.
beforeEach(() => {
  const values = new Map<string, string>()
  Object.defineProperty(window, 'localStorage', {
    configurable: true,
    value: {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => void values.set(key, value),
      removeItem: (key: string) => void values.delete(key),
      clear: () => values.clear()
    }
  })
})
afterEach(() => delete (window as { localStorage?: unknown }).localStorage)

describe('GraphRailGroup', () => {
  it('asks a project without a graph whether to set one up', async () => {
    const wrapper = group()

    await wrapper.find('[data-testid="graph-invite-accept"]').trigger('click')

    expect(wrapper.emitted('setup')).toHaveLength(1)
  })

  it('folds the question away for that project once declined, and keeps it folded', async () => {
    const wrapper = group()

    await wrapper.find('[data-testid="graph-invite-dismiss"]').trigger('click')

    expect(wrapper.find('[data-testid="graph-invite"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="rail-graphs-empty"]').exists()).toBe(true)
    expect(group().find('[data-testid="graph-invite"]').exists()).toBe(false)
  })

  it('lists the graphs with whether each can run, marking the one open', async () => {
    const stuck = aGraph({ id: 'g-2' as never, title: 'Draft', problem: 'never reaches it' })
    const wrapper = group({ graphs: [aGraph(), stuck], selectedId: aGraph().id })

    expect(wrapper.find('[data-testid="graph-invite"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="rail-graph-g-1"]').attributes('aria-current')).toBe('page')
    expect(wrapper.find('[data-testid="rail-graph-g-1"]').text()).toContain('2 steps · ready')
    expect(wrapper.find('[data-testid="rail-graph-g-2"]').text()).toContain("can't run yet")
    await wrapper.find('[data-testid="rail-graph-g-2"]').trigger('click')
    expect(wrapper.emitted('select')).toEqual([['g-2']])
  })
})
