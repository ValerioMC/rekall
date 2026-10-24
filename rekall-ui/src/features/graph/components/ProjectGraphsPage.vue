<script setup lang="ts">
import { computed, ref } from 'vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import DeploymentGraphPane from '@/features/graph/components/DeploymentGraphPane.vue'
import GraphRailGroup from '@/features/graph/components/GraphRailGroup.vue'
import GraphSetupPane from '@/features/graph/components/GraphSetupPane.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useDeploymentGraphsStore } from '@/features/graph/store/deploymentGraphs.store'
import type { Project } from '@/features/catalog/model/catalog'
import type { DeploymentGraph } from '@/features/graph/model/deployment-graph'
import type { DeploymentGraphId } from '@/model/branded'

/** The project's deployment graphs: their index beside the one open, or a new one being set up. */
const props = defineProps<{ project: Project; selectedId: DeploymentGraphId | null; settingUp: boolean }>()

const emit = defineEmits<{ open: [id: DeploymentGraphId | null]; setup: [] }>()

const store = useDeploymentGraphsStore()
const { run } = useAsyncAction()

const graphs = computed(() => store.graphsOf(props.project.id))
const selected = computed(() => graphs.value.find((graph) => graph.id === props.selectedId) ?? null)
const deleting = ref<DeploymentGraph | null>(null)

async function confirmDelete(): Promise<void> {
  const target = deleting.value
  deleting.value = null
  if (!target) return
  const removed = await run(() => store.remove(props.project.id, target.id).then(() => true), `Deleted ${target.title}`)
  if (removed && props.selectedId === target.id) emit('open', null)
}
</script>

<template>
  <div class="grid h-full min-h-0 grid-cols-1 gap-4 md:grid-cols-[232px_minmax(0,1fr)]" data-testid="graphs-page">
    <GraphRailGroup
      class="max-h-[30vh] overflow-y-auto md:max-h-none"
      :project-id="project.id"
      :graphs="graphs"
      :selected-id="selected?.id ?? null"
      :setting-up="settingUp"
      :loading="store.isLoading(project.id)"
      @select="emit('open', $event)"
      @setup="emit('setup')"
    />

    <Transition name="fade-quick" mode="out-in">
      <GraphSetupPane v-if="settingUp" key="graph-setup" :project="project" @created="emit('open', $event.id)" />
      <DeploymentGraphPane v-else-if="selected" :key="selected.id" :graph="selected" :project="project" @delete="deleting = selected" />
      <div
        v-else
        key="none"
        class="grid h-full min-h-[240px] place-items-center rounded-card border border-dashed border-border px-6 text-center"
        data-testid="graphs-none"
      >
        <p class="max-w-[44ch] text-small leading-relaxed text-text-subtle">
          {{ graphs.length ? 'Pick a graph to see its steps and routes.' : 'A graph is the route a commit takes to a running application. Start one with +.' }}
        </p>
      </div>
    </Transition>

    <Transition name="dialog">
      <AppConfirm
        v-if="deleting"
        :title="`Delete ${deleting.title}?`"
        body="The graph, its steps and its routes are removed. Tasks it was attached to keep the steps it laid on them."
        :blast="`Deletes the deployment graph “${deleting.title}” from ${project.title}.`"
        confirm-label="Delete graph"
        @cancel="deleting = null"
        @confirm="confirmDelete"
      />
    </Transition>
  </div>
</template>
