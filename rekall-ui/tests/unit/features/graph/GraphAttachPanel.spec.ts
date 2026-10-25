import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { createMemoryHistory, createRouter } from 'vue-router'
import GraphAttachPanel from '@/features/graph/components/GraphAttachPanel.vue'
import * as graphsApi from '@/features/graph/api/deployment-graphs.api'
import * as catalogApi from '@/features/catalog/api/catalog.api'
import { useConsoleStore } from '@/stores/console.store'
import type { Project, Task } from '@/features/catalog/model/catalog'
import type { CompanyId, TaskId } from '@/model/branded'
import { aGraph, PROJECT } from './graph.fixtures'

vi.mock('@/features/graph/api/deployment-graphs.api')
vi.mock('@/features/connection/api/connections.api')
vi.mock('@/features/catalog/api/catalog.api')

const task = { id: 't-1' as TaskId, projectId: PROJECT, title: 'Release 1', graphLaidAt: null, releaseSkipped: false } as unknown as Task

async function panel(hasSteps = false, laidOn: Task = task) {
  const store = useConsoleStore()
  store.projects = [{ id: PROJECT, companyId: 'c-1' as CompanyId, title: 'Vega' } as unknown as Project]
  const router = createRouter({ history: createMemoryHistory(), routes: [{ path: '/:p(.*)*', component: { template: '<div />' } }] })
  const wrapper = mount(GraphAttachPanel, { props: { task: laidOn, hasSteps }, global: { plugins: [router] } })
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
    vi.mocked(catalogApi.fetchTask).mockResolvedValue({ ...task, graphLaidAt: '2026-10-25T10:00:00Z' })
    const wrapper = await panel()

    await wrapper.find('[data-testid="graph-attach-run"]').trigger('click')
    await flushPromises()

    expect(graphsApi.attachDeploymentGraph).toHaveBeenCalledWith('t-1', 'g-1')
    expect(wrapper.emitted('attached')).toHaveLength(1)
  })

  it('announces nothing when the attach is refused', async () => {
    vi.mocked(graphsApi.fetchDeploymentGraphs).mockResolvedValue([aGraph()])
    vi.mocked(graphsApi.attachDeploymentGraph).mockRejectedValue(new Error('This task already has 2 steps.'))
    const wrapper = await panel()

    await wrapper.find('[data-testid="graph-attach-run"]').trigger('click')
    await flushPromises()

    expect(wrapper.emitted('attached')).toBeUndefined()
  })

  it('offers a graph that cannot run, without letting it be used', async () => {
    vi.mocked(graphsApi.fetchDeploymentGraphs).mockResolvedValue([aGraph({ problem: "never reaches 'Check'" })])
    const wrapper = await panel()

    expect(wrapper.find('[data-testid="graph-attach-run"]').attributes('disabled')).toBeDefined()
    expect(wrapper.text()).toContain("Can't run yet: never reaches 'Check'")
  })

  it('says where the graph goes: after the checklist, or after an implementation step', async () => {
    vi.mocked(graphsApi.fetchDeploymentGraphs).mockResolvedValue([aGraph()])

    expect((await panel(true)).find('[data-testid="graph-attach-where"]').text()).toContain('after this checklist')
    expect((await panel(false)).find('[data-testid="graph-attach-where"]').text()).toContain('An implementation step goes first')
  })

  it("says the project's release graph is laid when the task runs, and leaves the task unreleased on request", async () => {
    vi.mocked(graphsApi.fetchDeploymentGraphs).mockResolvedValue([aGraph({ title: 'Release', releaseDefault: true })])
    vi.mocked(graphsApi.skipRelease).mockResolvedValue({ ...task, releaseSkipped: true })
    const wrapper = await panel()

    expect(wrapper.get('[data-testid="graph-release-on-run"]').text()).toContain('Release is laid on its own when you run this task')
    await wrapper.get('[data-testid="graph-release-toggle"]').trigger('click')
    await flushPromises()

    expect(graphsApi.skipRelease).toHaveBeenCalledWith('t-1', true)
  })

  it('says a task left unreleased lays no graph when run, and offers to release it again', async () => {
    vi.mocked(graphsApi.fetchDeploymentGraphs).mockResolvedValue([aGraph({ releaseDefault: true })])
    const wrapper = await panel(false, { ...task, releaseSkipped: true })

    expect(wrapper.get('[data-testid="graph-release-on-run"]').text()).toContain('Left unreleased')
    expect(wrapper.get('[data-testid="graph-release-toggle"]').text()).toBe('Release it when run')
  })

  it('says nothing about running when the project chose no release graph', async () => {
    vi.mocked(graphsApi.fetchDeploymentGraphs).mockResolvedValue([aGraph()])

    expect((await panel()).find('[data-testid="graph-release-on-run"]').exists()).toBe(false)
  })

  it('points a project without a graph to its page', async () => {
    vi.mocked(graphsApi.fetchDeploymentGraphs).mockResolvedValue([])
    const wrapper = await panel()

    expect(wrapper.find('[data-testid="graph-attach-none"]').text()).toContain('Vega has no deployment graph yet')
    expect(wrapper.find('a').attributes('href')).toBe('/projects/p-1')
  })
})
