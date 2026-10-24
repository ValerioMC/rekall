import { defineStore } from 'pinia'
import { ref } from 'vue'
import {
  createDeploymentGraph,
  deleteDeploymentGraph,
  fetchDeploymentGraphs,
  replaceDeploymentGraph
} from '@/features/graph/api/deployment-graphs.api'
import type { DeploymentGraph, GraphDraft } from '@/features/graph/model/deployment-graph'
import type { DeploymentGraphId, ProjectId } from '@/model/branded'

/** The deployment graphs of the projects the console has opened, oldest first. */
export const useDeploymentGraphsStore = defineStore('deploymentGraphs', () => {
  const byProject = ref<Readonly<Record<string, readonly DeploymentGraph[]>>>({})
  const loading = ref<ReadonlySet<string>>(new Set())

  function graphsOf(projectId: ProjectId): readonly DeploymentGraph[] {
    return byProject.value[projectId] ?? []
  }

  function isLoaded(projectId: ProjectId): boolean {
    return projectId in byProject.value
  }

  function isLoading(projectId: ProjectId): boolean {
    return loading.value.has(projectId)
  }

  function put(projectId: ProjectId, graphs: readonly DeploymentGraph[]): void {
    byProject.value = { ...byProject.value, [projectId]: graphs }
  }

  async function load(projectId: ProjectId): Promise<void> {
    loading.value = new Set(loading.value).add(projectId)
    try {
      put(projectId, await fetchDeploymentGraphs(projectId))
    } finally {
      const rest = new Set(loading.value)
      rest.delete(projectId)
      loading.value = rest
    }
  }

  async function ensureLoaded(projectId: ProjectId): Promise<void> {
    if (!isLoaded(projectId) && !isLoading(projectId)) await load(projectId)
  }

  async function create(projectId: ProjectId, draft: GraphDraft): Promise<DeploymentGraph> {
    const created = await createDeploymentGraph(projectId, draft)
    put(projectId, [...graphsOf(projectId), created])
    return created
  }

  async function replace(id: DeploymentGraphId, draft: GraphDraft): Promise<DeploymentGraph> {
    const saved = await replaceDeploymentGraph(id, draft)
    put(saved.projectId, graphsOf(saved.projectId).map((known) => (known.id === saved.id ? saved : known)))
    return saved
  }

  async function remove(projectId: ProjectId, id: DeploymentGraphId): Promise<void> {
    await deleteDeploymentGraph(id)
    put(projectId, graphsOf(projectId).filter((graph) => graph.id !== id))
  }

  return { graphsOf, isLoaded, isLoading, load, ensureLoaded, create, replace, remove }
})
