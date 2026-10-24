import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { createMemoryHistory, createRouter } from 'vue-router'
import GraphAttachPanel from '@/features/graph/components/GraphAttachPanel.vue'
import * as graphsApi from '@/features/graph/api/deployment-graphs.api'
import { useConsoleStore } from '@/stores/console.store'
import type { Project, Task } from '@/features/catalog/model/catalog'
import type { CompanyId, TaskId } from '@/model/branded'
import { aGraph, PROJECT } from './graph.fixtures'

vi.mock('@/features/graph/api/deployment-graphs.api')
vi.mock('@/features/connection/api/connections.api')

const task = { id: 't-1' as TaskId, projectId: PROJECT, title: 'Release 1' } as unknown as Task

async function panel() {
  const store = useConsoleStore()
  store.projects = [{ id: PROJECT, companyId: 'c-1' as CompanyId, title: 'Vega' } as unknown as Project]
  const router = createRouter({ history: createMemoryHistory(), routes: [{ path: '/:p(.*)*', component: { template: '<div />' } }] })
  const wrapper = mount(GraphAttachPanel, { props: { task }, global: { plugins: [router] } })
  await flushPromises()
  return wrapper
}

beforeEach(() => {
  setActivePinia(createPinia())
  vi.resetAllMocks()
})

describe('GraphAttachPanel', () => {
  it("lays a graph the project can run on the task", async () => {
    vi.mocked(graphsApi.fetchDeploymentGraphs).mockResolvedValue([aGraph()])
    vi.mocked(graphsApi.attachDeploymentGraph).mockResolvedValue([])
    const wrapper = await panel()

    await wrapper.find('[data-testid="graph-attach-run"]').trigger('click')
    await flushPromises()

    expect(graphsApi.attachDeploymentGraph).toHaveBeenCalledWith('t-1', 'g-1')
  })

  it('offers a graph that cannot run, without letting it be used', async () => {
    vi.mocked(graphsApi.fetchDeploymentGraphs).mockResolvedValue([aGraph({ problem: "never reaches 'Check'" })])
    const wrapper = await panel()

    expect(wrapper.find('[data-testid="graph-attach-run"]').attributes('disabled')).toBeDefined()
    expect(wrapper.text()).toContain("Can't run yet: never reaches 'Check'")
  })

  it('points a project without a graph to its page', async () => {
    vi.mocked(graphsApi.fetchDeploymentGraphs).mockResolvedValue([])
    const wrapper = await panel()

    expect(wrapper.find('[data-testid="graph-attach-none"]').text()).toContain('Vega has no deployment graph yet')
    expect(wrapper.find('a').attributes('href')).toBe('/projects/p-1')
  })
})
