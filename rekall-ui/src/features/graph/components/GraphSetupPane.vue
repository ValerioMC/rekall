<script setup lang="ts">
import { computed, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import { useDeploymentGraphsStore } from '@/features/graph/store/deploymentGraphs.store'
import { chainOf, loopsBack } from '@/features/graph/lib/graph-chain'
import { blankGraph, releaseLoop } from '@/features/graph/lib/graph-presets'
import type { DeploymentGraph, GraphDraft } from '@/features/graph/model/deployment-graph'
import type { Project } from '@/features/catalog/model/catalog'
import type { ConnectionId } from '@/model/branded'
import GraphStepChain from './GraphStepChain.vue'

/**
 * Where a project's deployment graph starts: the full release loop, every step already on the
 * project's connection of its kind, or one blank step to draw from. Either opens in the editor.
 */
const props = defineProps<{ project: Project }>()

const emit = defineEmits<{ created: [graph: DeploymentGraph] }>()

type Template = 'release' | 'blank'

const graphs = useDeploymentGraphsStore()
const connections = useConnectionsStore()
const { run, isRunning } = useAsyncAction()

const bindings = computed(() => connections.projectConnectionsOf(props.project.id))
const template = ref<Template>('release')
const title = ref(freeGraphTitle('Release'))

function freeGraphTitle(base: string): string {
  const taken = new Set(graphs.graphsOf(props.project.id).map((graph) => graph.title.toLowerCase()))
  if (!taken.has(base.toLowerCase())) return base
  let number = 2
  while (taken.has(`${base} ${number}`.toLowerCase())) number += 1
  return `${base} ${number}`
}

function kindOf(id: ConnectionId) {
  return bindings.value.find((binding) => binding.connectionId === id)?.kind ?? null
}

const release = computed(() => releaseLoop(title.value.trim(), bindings.value))
const missing = computed(() => release.value.steps.filter((step) => step.connectionId === null).map((step) => step.title))
const draft = computed<GraphDraft>(() => (template.value === 'release' ? release.value : blankGraph(title.value.trim())))

async function create(): Promise<void> {
  if (!title.value.trim()) return
  const created = await run(() => graphs.create(props.project.id, draft.value))
  if (created) emit('created', created)
}
</script>

<template>
  <section
    class="flex h-full min-h-0 flex-col overflow-y-auto rounded-card border border-border bg-surface p-6 shadow-[var(--shadow-raised)]"
    aria-label="New deployment graph"
    data-testid="graph-setup"
  >
    <div class="mx-auto w-full max-w-[640px]">
      <p class="eyebrow">New deployment graph</p>
      <h2 class="mt-1 text-heading font-semibold tracking-[-0.02em] text-text">Release this project the same way every time</h2>
      <p class="mt-2 text-body leading-relaxed text-text-muted">
        A deployment graph is the route the work takes from a commit to a running application, with every step
        reaching the system it needs. Attach it to a task and a session runs it end to end, going back to fix
        what fails, without stopping to ask.
      </p>

      <form class="mt-6 flex flex-col gap-5" @submit.prevent="create">
        <div role="radiogroup" aria-label="Start from" class="grid gap-3 sm:grid-cols-2">
          <button
            type="button"
            role="radio"
            :aria-checked="template === 'release'"
            class="focus-ring flex cursor-pointer flex-col gap-3 rounded-card border p-4 text-left transition-colors sm:col-span-2"
            :class="template === 'release' ? 'border-accent/60 bg-accent-soft' : 'border-border hover:bg-surface-raised'"
            data-testid="graph-template-release"
            @click="template = 'release'"
          >
            <span>
              <span class="block text-title font-medium text-text">Full release loop</span>
              <span class="mt-0.5 block text-small text-text-muted">
                Push the commit, follow the pipeline, deploy, check. A failure goes back to the push to be fixed,
                at most three times.
              </span>
            </span>
            <GraphStepChain :links="chainOf(release.steps, kindOf)" :loops="loopsBack(release.steps)" />
            <span v-if="missing.length" class="text-caption text-warn" data-testid="graph-template-missing">
              No connection yet for {{ missing.join(', ') }}: bind a GitLab instance or a cluster under Connections, or
              pick one per step later.
            </span>
          </button>
          <button
            type="button"
            role="radio"
            :aria-checked="template === 'blank'"
            class="focus-ring flex cursor-pointer flex-col gap-1 rounded-card border p-4 text-left transition-colors"
            :class="template === 'blank' ? 'border-accent/60 bg-accent-soft' : 'border-border hover:bg-surface-raised'"
            data-testid="graph-template-blank"
            @click="template = 'blank'"
          >
            <span class="text-title font-medium text-text">Blank graph</span>
            <span class="text-small text-text-muted">One step, to draw the rest yourself.</span>
          </button>
        </div>

        <div class="flex flex-col gap-1.5">
          <label for="graph-setup-title" class="eyebrow">Name</label>
          <input
            id="graph-setup-title"
            v-model="title"
            maxlength="200"
            autocomplete="off"
            class="field h-(--spacing-control) w-full rounded-control px-3 text-body text-text"
            data-testid="graph-setup-title"
          />
        </div>

        <AppButton
          type="submit"
          variant="primary"
          class="self-start"
          :disabled="!title.trim()"
          :loading="isRunning"
          data-testid="graph-setup-create"
        >
          Create graph
        </AppButton>
      </form>
    </div>
  </section>
</template>
