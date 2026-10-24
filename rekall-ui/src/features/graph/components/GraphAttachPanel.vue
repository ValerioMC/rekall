<script setup lang="ts">
import { computed, watch } from 'vue'
import { RouterLink } from 'vue-router'
import AppButton from '@/components/ui/AppButton.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useConsoleStore } from '@/stores/console.store'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import { useDeploymentGraphsStore } from '@/features/graph/store/deploymentGraphs.store'
import { draftOf } from '@/features/graph/lib/graph-draft'
import { chainOf, loopsBack } from '@/features/graph/lib/graph-chain'
import type { ConnectionKind } from '@/features/connection/model/connection'
import type { DeploymentGraph } from '@/features/graph/model/deployment-graph'
import type { Task } from '@/features/catalog/model/catalog'
import type { ConnectionId } from '@/model/branded'
import GraphStepChain from './GraphStepChain.vue'

/**
 * What a task with no steps can run: its project's deployment graphs, each laid on the task in one
 * click as a flow a session works through unattended. A graph that cannot run yet says why.
 */
const props = defineProps<{ task: Task }>()

const emit = defineEmits<{ attached: [] }>()

const store = useConsoleStore()
const graphs = useDeploymentGraphsStore()
const connections = useConnectionsStore()
const { run, isRunning } = useAsyncAction()

const project = computed(() => store.projects.find((candidate) => candidate.id === props.task.projectId) ?? null)
const available = computed(() => graphs.graphsOf(props.task.projectId))

watch(
  project,
  (current) => {
    if (!current) return
    void run(() => graphs.ensureLoaded(current.id))
    void run(() => connections.ensureLoaded(current.companyId))
  },
  { immediate: true }
)

function kindOf(id: ConnectionId): ConnectionKind | null {
  return connections.connectionById(id)?.kind ?? null
}

function chain(graph: DeploymentGraph) {
  const steps = draftOf(graph).steps
  return { links: chainOf(steps, kindOf), loops: loopsBack(steps) }
}

async function attach(graph: DeploymentGraph): Promise<void> {
  const laid = await run(() => store.attachGraph(props.task.id, graph.id).then(() => true), `${graph.title} laid on the task`)
  if (laid) emit('attached')
}
</script>

<template>
  <section class="mb-7 max-w-[640px]" aria-labelledby="graph-attach-heading" data-testid="graph-attach">
    <h3 id="graph-attach-heading" class="eyebrow">Run a deployment graph</h3>
    <ul v-if="available.length" class="mt-2 flex flex-col gap-2">
      <li
        v-for="graph in available"
        :key="graph.id"
        class="flex flex-col gap-2.5 rounded-card border border-border bg-surface p-3.5 shadow-[var(--shadow-raised)]"
        :data-testid="`graph-attach-${graph.id}`"
      >
        <div class="flex items-start gap-3">
          <div class="min-w-0 flex-1">
            <p class="truncate text-title font-medium text-text">{{ graph.title }}</p>
            <p v-if="graph.description" class="mt-0.5 line-clamp-2 text-small text-text-muted">{{ graph.description }}</p>
          </div>
          <AppButton
            size="sm"
            variant="primary"
            :disabled="graph.problem !== null"
            :loading="isRunning"
            data-testid="graph-attach-run"
            @click="attach(graph)"
          >
            Use this graph
          </AppButton>
        </div>
        <GraphStepChain v-bind="chain(graph)" />
        <p v-if="graph.problem" class="text-caption text-warn">Can't run yet: {{ graph.problem }}</p>
      </li>
    </ul>
    <p v-else-if="project && graphs.isLoaded(project.id)" class="mt-1.5 text-small leading-relaxed text-text-muted" data-testid="graph-attach-none">
      {{ project.title }} has no deployment graph yet.
      <RouterLink :to="`/projects/${project.id}`" class="text-anchor hover:underline">Set one up on the project page</RouterLink>
      to release a task with push, build, deploy and check in one go.
    </p>
  </section>
</template>
