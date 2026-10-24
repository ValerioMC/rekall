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

async function pane(): Promise<VueWrapper> {
  const store = useDeploymentGraphsStore()
  vi.mocked(graphsApi.fetchDeploymentGraphs).mockResolvedValue([aGraph()])
  await store.load(PROJECT)
  wrapper = mount(DeploymentGraphPane, { props: { graph: aGraph(), project }, attachTo: document.body })
  return wrapper
}

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
})
