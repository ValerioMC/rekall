<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import { loadPersistedSet, savePersistedSet } from '@/common/config/persisted-set'
import type { DeploymentGraph } from '@/features/graph/model/deployment-graph'
import type { DeploymentGraphId, ProjectId } from '@/model/branded'

/**
 * The project's deployment graphs in its pages rail. A project without one is asked whether to set
 * one up; declining folds the question away for this project, in this browser.
 */
const props = defineProps<{
  projectId: ProjectId
  graphs: readonly DeploymentGraph[]
  selectedId: DeploymentGraphId | null
  settingUp: boolean
  loading: boolean
}>()

const emit = defineEmits<{ select: [id: DeploymentGraphId]; setup: [] }>()

const DISMISSED_KEY = 'rekall.graph-invite-dismissed'

const dismissed = ref<ReadonlySet<string>>(loadPersistedSet<string>(DISMISSED_KEY))
const inviting = computed(
  () => !props.loading && !props.graphs.length && !props.settingUp && !dismissed.value.has(props.projectId)
)

function dismiss(): void {
  dismissed.value = new Set(dismissed.value).add(props.projectId)
  savePersistedSet(DISMISSED_KEY, dismissed.value)
}

watch(
  () => props.projectId,
  () => (dismissed.value = loadPersistedSet<string>(DISMISSED_KEY))
)

function caption(graph: DeploymentGraph): string {
  const steps = graph.steps.length === 1 ? '1 step' : `${graph.steps.length} steps`
  return graph.problem ? `${steps} · can't run yet` : `${steps} · ready`
}
</script>

<template>
  <div class="mt-3 flex flex-col gap-1" data-testid="rail-graphs">
    <div class="flex items-center justify-between pl-4 pr-1">
      <p class="eyebrow">
        Deployment graphs<span v-if="graphs.length" class="ml-1.5 text-text-subtle">{{ graphs.length }}</span>
      </p>
      <button
        type="button"
        class="key-quiet focus-ring grid size-6 place-items-center rounded-key"
        aria-label="New deployment graph"
        title="New deployment graph"
        data-testid="rail-new-graph"
        @click="emit('setup')"
      >
        <svg class="size-3.5" viewBox="0 0 14 14" fill="none" aria-hidden="true">
          <path d="M7 2.5v9M2.5 7h9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
      </button>
    </div>

    <div v-if="inviting" class="mx-1 rounded-card border border-accent/30 bg-accent-soft p-3" data-testid="graph-invite">
      <svg class="mb-2 h-5 w-24 text-accent" viewBox="0 0 96 20" fill="none" aria-hidden="true">
        <circle cx="8" cy="10" r="4" stroke="currentColor" stroke-width="1.4" />
        <circle cx="34" cy="10" r="4" stroke="currentColor" stroke-width="1.4" />
        <circle cx="60" cy="10" r="4" stroke="currentColor" stroke-width="1.4" />
        <circle cx="86" cy="10" r="4" stroke="currentColor" stroke-width="1.4" />
        <path d="M12 10h18M38 10h18M64 10h18" stroke="currentColor" stroke-width="1.4" />
        <path d="M86 14c0 5-78 5-78 0" stroke="currentColor" stroke-width="1.2" stroke-dasharray="3 3" />
      </svg>
      <p class="text-small font-medium text-text">No deployment graph yet</p>
      <p class="mt-1 text-caption leading-relaxed text-text-muted">
        Set up push, build, deploy and check, each on its connection, so a task can be released in one go?
      </p>
      <div class="mt-2.5 flex flex-wrap gap-1.5">
        <AppButton size="sm" variant="primary" data-testid="graph-invite-accept" @click="emit('setup')">Set it up</AppButton>
        <AppButton size="sm" variant="ghost" data-testid="graph-invite-dismiss" @click="dismiss">Not now</AppButton>
      </div>
    </div>

    <ul class="flex flex-col gap-1">
      <li v-if="settingUp">
        <span class="selected-row flex items-center rounded-control py-2 pl-4 pr-3 text-body text-text" aria-current="page">
          New graph
        </span>
      </li>
      <li v-for="graph in graphs" :key="graph.id">
        <button
          type="button"
          class="focus-ring flex w-full items-center gap-2.5 rounded-control py-2 pl-4 pr-3 text-left transition-colors"
          :class="selectedId === graph.id ? 'selected-row' : 'hover:bg-surface-raised'"
          :aria-current="selectedId === graph.id ? 'page' : undefined"
          :data-testid="`rail-graph-${graph.id}`"
          @click="emit('select', graph.id)"
        >
          <span class="min-w-0 flex-1">
            <span class="block truncate text-body text-text">{{ graph.title }}</span>
            <span class="block truncate text-caption" :class="graph.problem ? 'text-warn' : 'text-text-subtle'">
              {{ caption(graph) }}
            </span>
          </span>
        </button>
      </li>
    </ul>
    <p
      v-if="!graphs.length && !inviting && !settingUp && !loading"
      class="px-4 text-caption text-text-subtle"
      data-testid="rail-graphs-empty"
    >
      None yet.
    </p>
  </div>
</template>
