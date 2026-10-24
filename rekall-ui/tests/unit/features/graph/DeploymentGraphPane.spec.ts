import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import DeploymentGraphPane from '@/features/graph/components/DeploymentGraphPane.vue'
import * as graphsApi from '@/features/graph/api/deployment-graphs.api'
import { useDeploymentGraphsStore } from '@/features/graph/store/deploymentGraphs.store'
import { ApiError } from '@/api/client'
import type { Project } from '@/features/catalog/model/catalog'
import { aGraph, PROJECT } from './graph.fixtures'

vi.mock('@/features/graph/api/deployment-graphs.api')

const project = { id: PROJECT, companyId: 'c-1', title: 'Vega' } as unknown as Project

let wrapper: VueWrapper | null = null

async function pane(graph = aGraph()): Promise<VueWrapper> {
  const store = useDeploymentGraphsStore()
  vi.mocked(graphsApi.fetchDeploymentGraphs).mockResolvedValue([graph])
  await store.load(PROJECT)
  wrapper = mount(DeploymentGraphPane, { props: { graph, project }, attachTo: document.body })
  return wrapper
}

/** Where an absolutely placed element sits on the canvas, from its inline style. */
function at(element: Element): { left: number; top: number } {
  const style = element.getAttribute('style') ?? ''
  return {
    left: Number(/left:\s*(-?[\d.]+)px/.exec(style)?.[1]),
    top: Number(/top:\s*(-?[\d.]+)px/.exec(style)?.[1])
  }
}

/** A step's box: the element its node button sits in. */
function boxOf(node: { element: Element }): { left: number; top: number } {
  return at(node.element.parentElement!)
}

const HALF_NODE = 116

beforeEach(() => {
  setActivePinia(createPinia())
  vi.resetAllMocks()
  vi.useFakeTimers()
})

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
  vi.useRealTimers()
})

describe('DeploymentGraphPane', () => {
  it('draws every step and opens the first in the inspector', async () => {
    const view = await pane()

    expect(view.findAll('[data-testid="graph-node"]')).toHaveLength(2)
    expect((view.find('[data-testid="graph-step-title"]').element as HTMLInputElement).value).toBe('Push commit')
    expect(view.find('[data-testid="graph-readiness"]').text()).toBe('Ready to attach')
  })

  it('saves the whole graph a moment after an edit, steps named by their keys', async () => {
    vi.mocked(graphsApi.replaceDeploymentGraph).mockImplementation(async () => aGraph({ title: 'Ship' }))
    const view = await pane()

    await view.find('[data-testid="graph-title"]').setValue('Ship')
    expect(graphsApi.replaceDeploymentGraph).not.toHaveBeenCalled()
    await vi.advanceTimersByTimeAsync(800)
    await flushPromises()

    const [id, draft] = vi.mocked(graphsApi.replaceDeploymentGraph).mock.calls[0]!
    expect(id).toBe('g-1')
    expect(draft.title).toBe('Ship')
    expect(draft.steps[1]!.routes).toEqual([{ outcome: 'healthy', toKey: null }, { outcome: 'error', toKey: 's-1' }])
    expect(view.find('[data-testid="graph-save-state"]').text()).toBe('Saved')
  })

  it('keeps a refused edit on screen with the reason', async () => {
    vi.mocked(graphsApi.replaceDeploymentGraph).mockRejectedValue(new ApiError(400, "The step 'Check' appears twice.", 'x'))
    const view = await pane()

    await view.find('[data-testid="graph-step-title"]').setValue('Check')
    await vi.advanceTimersByTimeAsync(800)
    await flushPromises()

    expect(view.find('[data-testid="graph-refusal"]').text()).toContain('appears twice')
    expect(view.find('[data-testid="graph-save-state"]').text()).toBe('Not saved')
    expect((view.find('[data-testid="graph-step-title"]').element as HTMLInputElement).value).toBe('Check')
  })

  it('adds a preset after the picked step, routed from it when it routed nowhere', async () => {
    const view = await pane()
    await view.findAll('[data-testid="graph-node"]')[1]!.trigger('click')

    await view.find('[data-testid="graph-add-deploy"]').trigger('click')

    const titles = view.findAll('[data-testid="graph-node"]').map((node) => node.attributes('aria-label'))
    expect(titles).toHaveLength(3)
    expect(titles[2]).toContain('Deploy')
    expect((view.find('[data-testid="graph-step-title"]').element as HTMLInputElement).value).toBe('Deploy')
  })

  it('draws a step where it was dragged, with the routes that touch it following it', async () => {
    const base = aGraph()
    const placed = aGraph({ steps: [base.steps[0]!, { ...base.steps[1]!, flowX: 600, flowY: 500 }] })
    const view = await pane(placed)

    const [push, check] = view.findAll('[data-testid="graph-node"]').map(boxOf)
    expect(check).toEqual({ left: 600, top: 500 })
    const pushed = view.findAll('[data-testid="graph-outcome"]').find((label) => label.text() === 'pushed')!
    expect(at(pushed.element).left).toBe((push!.left + HALF_NODE + 600 + HALF_NODE) / 2)
  })

  it('saves where a dragged step was dropped', async () => {
    vi.mocked(graphsApi.replaceDeploymentGraph).mockImplementation(async () => aGraph())
    const view = await pane()
    const node = view.findAll('[data-testid="graph-node"]')[1]!
    const origin = boxOf(node)

    await node.trigger('pointerdown', { button: 0, clientX: 100, clientY: 100 })
    window.dispatchEvent(new PointerEvent('pointermove', { clientX: 220, clientY: 160 }))
    window.dispatchEvent(new PointerEvent('pointerup', { clientX: 220, clientY: 160 }))
    await vi.advanceTimersByTimeAsync(800)
    await flushPromises()

    const [, draft] = vi.mocked(graphsApi.replaceDeploymentGraph).mock.calls[0]!
    expect(draft.steps[1]).toMatchObject({ flowX: origin.left + 120, flowY: origin.top + 60 })
    expect(draft.steps[0]).toMatchObject({ flowX: null, flowY: null })
  })
})
