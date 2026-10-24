<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import { useDeploymentGraphsStore } from '@/features/graph/store/deploymentGraphs.store'
import { isFlowEnd, type RouteTarget } from '@/features/flow/lib/route-target'
import {
  addRoute,
  clearPlaces,
  draftOf,
  freeOutcome,
  insertStep,
  moveStep,
  removeRoute,
  removeStep,
  sameDraft,
  updateRoute,
  updateStep,
  type Direction
} from '@/features/graph/lib/graph-draft'
import { freeTitle, stepFrom, type StepPreset } from '@/features/graph/lib/graph-presets'
import { GRAPH_STEPS_MAX, type DeploymentGraph, type DraftRoute, type DraftStep, type GraphDraft } from '@/features/graph/model/deployment-graph'
import type { Point } from '@/features/diagram/lib/viewport'
import type { Project } from '@/features/catalog/model/catalog'
import GraphCanvas from './GraphCanvas.vue'
import GraphStepInspector from './GraphStepInspector.vue'
import GraphStepPalette from './GraphStepPalette.vue'

/**
 * A deployment graph being edited: its name, the canvas the steps are drawn on and the picked
 * step's inspector. Every change is saved on its own a moment later; a change the server refuses
 * stays here with the reason beside the title, and the next edit tries again.
 */
const props = defineProps<{ graph: DeploymentGraph; project: Project }>()

const emit = defineEmits<{ delete: [] }>()

const SAVE_DELAY_MS = 700

type SaveState = 'saved' | 'pending' | 'saving' | 'refused'

const graphs = useDeploymentGraphsStore()
const connections = useConnectionsStore()
const bindings = computed(() => connections.projectConnectionsOf(props.project.id))

const draft = ref<GraphDraft>(draftOf(props.graph))
let sent: GraphDraft = snapshot()
const saveState = ref<SaveState>('saved')
const refusal = ref<string | null>(null)
let timer: ReturnType<typeof setTimeout> | null = null

const selectedKey = ref<string | null>(draft.value.steps[0]?.key ?? null)
const selectedIndex = computed(() => draft.value.steps.findIndex((step) => step.key === selectedKey.value))
const selected = computed<DraftStep | null>(() => draft.value.steps[selectedIndex.value] ?? null)
const focusRoute = ref<{ key: string; index: number } | null>(null)

watch(
  () => props.graph.id,
  () => {
    flush()
    draft.value = draftOf(props.graph)
    sent = snapshot()
    saveState.value = 'saved'
    refusal.value = null
    selectedKey.value = draft.value.steps[0]?.key ?? null
  }
)

watch(
  draft,
  (current) => {
    if (sameDraft(current, sent)) return
    saveState.value = 'pending'
    if (timer) clearTimeout(timer)
    timer = setTimeout(() => void save(), SAVE_DELAY_MS)
  },
  { deep: true }
)

/** A copy of the draft as it stands: the fields edit it in place, so the last one sent is kept apart. */
function snapshot(): GraphDraft {
  return JSON.parse(JSON.stringify(draft.value)) as GraphDraft
}

async function save(): Promise<void> {
  timer = null
  const wanted = snapshot()
  saveState.value = 'saving'
  try {
    await graphs.replace(props.graph.id, wanted)
    sent = wanted
    refusal.value = null
    saveState.value = sameDraft(draft.value, wanted) ? 'saved' : 'pending'
  } catch (error) {
    refusal.value = error instanceof Error ? error.message : String(error)
    saveState.value = 'refused'
  }
}

function flush(): void {
  if (!timer) return
  clearTimeout(timer)
  void save()
}

onUnmounted(flush)

const SAVE_LABEL: Readonly<Record<SaveState, string>> = {
  saved: 'Saved',
  pending: 'Unsaved',
  saving: 'Saving',
  refused: 'Not saved'
}

function change(next: GraphDraft): void {
  draft.value = next
}

function addPreset(preset: StepPreset): void {
  const key = crypto.randomUUID()
  const step = { ...stepFrom(preset, key, bindings.value), title: freeTitle(preset.title, draft.value.steps) }
  change(insertStep(draft.value, step, selectedKey.value))
  selectedKey.value = key
}

/** A route drawn on the canvas gets a free outcome, focused in the inspector to be named. */
function connect(from: string, to: RouteTarget<string>): void {
  const step = draft.value.steps.find((candidate) => candidate.key === from)
  if (!step) return
  const route: DraftRoute = { outcome: freeOutcome(step, isFlowEnd(to)), toKey: isFlowEnd(to) ? null : to }
  change(addRoute(draft.value, from, route))
  selectedKey.value = from
  focusRoute.value = { key: from, index: step.routes.length }
}

/** Where a dragged step was dropped; it stays there until it is dragged again. */
function place(key: string, at: Point): void {
  change(updateStep(draft.value, key, { flowX: Math.round(at.x), flowY: Math.round(at.y) }))
}

function addEmptyRoute(): void {
  const step = selected.value
  if (!step) return
  connect(step.key, 'flow:end')
}

function update(patch: Partial<Omit<DraftStep, 'key'>>): void {
  if (selectedKey.value) change(updateStep(draft.value, selectedKey.value, patch))
}

function editRoute(index: number, patch: Partial<DraftRoute>): void {
  if (selectedKey.value) change(updateRoute(draft.value, selectedKey.value, index, patch))
}

function dropRoute(index: number): void {
  if (selectedKey.value) change(removeRoute(draft.value, selectedKey.value, index))
}

function move(direction: Direction): void {
  if (selectedKey.value) change(moveStep(draft.value, selectedKey.value, direction))
}

function dropStep(): void {
  const key = selectedKey.value
  if (!key) return
  const index = selectedIndex.value
  change(removeStep(draft.value, key))
  selectedKey.value = draft.value.steps[Math.max(0, index - 1)]?.key ?? null
}

const focusIndex = computed(() =>
  focusRoute.value && focusRoute.value.key === selectedKey.value ? focusRoute.value.index : null
)
</script>

<template>
  <section
    class="flex h-full min-h-0 flex-col overflow-hidden rounded-card border border-border bg-surface shadow-[var(--shadow-raised)]"
    aria-label="Deployment graph"
    data-testid="graph-pane"
  >
    <header class="shrink-0 border-b border-border px-5 py-3">
      <div class="flex items-start gap-3">
        <div class="min-w-0 flex-1">
          <label class="sr-only" for="graph-title">Graph title</label>
          <input
            id="graph-title"
            v-model="draft.title"
            maxlength="200"
            autocomplete="off"
            placeholder="Name the graph"
            class="focus-ring w-full rounded-control bg-transparent text-title font-semibold tracking-[-0.01em] text-text outline-none placeholder:text-text-subtle"
            data-testid="graph-title"
          />
          <label class="sr-only" for="graph-description">What the graph is for</label>
          <input
            id="graph-description"
            v-model="draft.description"
            maxlength="10000"
            autocomplete="off"
            placeholder="What this graph is for"
            class="focus-ring mt-0.5 w-full rounded-control bg-transparent text-small text-text-muted outline-none placeholder:text-text-subtle"
            data-testid="graph-description"
          />
        </div>
        <div class="flex shrink-0 items-center gap-2">
          <AppBadge v-if="graph.problem" tone="warn" dot :title="graph.problem" data-testid="graph-readiness">
            Can't run yet
          </AppBadge>
          <AppBadge v-else tone="safe" dot data-testid="graph-readiness">Ready to attach</AppBadge>
          <span
            class="flex items-center gap-1.5 text-small"
            :class="saveState === 'refused' ? 'text-danger' : saveState === 'saved' ? 'text-text-subtle' : 'text-warn'"
            role="status"
            data-testid="graph-save-state"
          >
            <span
              class="size-1.5 rounded-full"
              :class="{
                'bg-safe': saveState === 'saved',
                'bg-warn': saveState === 'pending',
                'animate-pulse bg-accent': saveState === 'saving',
                'bg-danger': saveState === 'refused'
              }"
              aria-hidden="true"
            />
            {{ SAVE_LABEL[saveState] }}
          </span>
          <button
            type="button"
            class="focus-ring grid size-8 cursor-pointer place-items-center rounded-control text-text-subtle transition-colors hover:bg-danger-soft hover:text-danger"
            :aria-label="`Delete ${graph.title}`"
            title="Delete graph"
            data-testid="graph-delete"
            @click="emit('delete')"
          >
            <svg class="size-4" viewBox="0 0 14 14" fill="none" aria-hidden="true">
              <path d="M2.5 4h9M5.5 4V2.5h3V4M4 4l.6 7.5h4.8L10 4" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
          </button>
        </div>
      </div>
      <p v-if="refusal" class="mt-2 rounded-control bg-danger-soft px-3 py-1.5 text-small text-danger" role="alert" data-testid="graph-refusal">
        {{ refusal }}
      </p>
      <p
        v-else-if="graph.problem && saveState === 'saved'"
        class="mt-2 rounded-control bg-warn-soft px-3 py-1.5 text-small text-warn"
        data-testid="graph-problem"
      >
        {{ graph.problem }}
      </p>
    </header>

    <div class="flex shrink-0 items-center justify-between gap-3 border-b border-border px-4 py-2">
      <GraphStepPalette :disabled="draft.steps.length >= GRAPH_STEPS_MAX" @add="addPreset" />
      <span class="hidden text-caption text-text-subtle xl:inline">Drag a step to move it, its handle onto another to route it</span>
    </div>

    <div class="grid min-h-0 flex-1 grid-cols-1 md:grid-cols-[minmax(0,1fr)_340px]">
      <GraphCanvas
        class="min-h-[320px]"
        :steps="draft.steps"
        :bindings="bindings"
        :selected-key="selectedKey"
        @select="selectedKey = $event"
        @connect="connect"
        @place="place"
        @tidy="change(clearPlaces(draft))"
      />
      <aside class="min-h-0 overflow-y-auto border-t border-border md:border-l md:border-t-0" aria-label="Picked step">
        <GraphStepInspector
          v-if="selected"
          :key="selected.key"
          :step="selected"
          :position="selectedIndex"
          :steps="draft.steps"
          :bindings="bindings"
          :focus-route="focusIndex"
          @update="update"
          @add-route="addEmptyRoute"
          @update-route="editRoute"
          @remove-route="dropRoute"
          @move="move"
          @remove="dropStep"
        />
        <p v-else class="p-4 text-small text-text-subtle">Pick a step on the canvas to set it up.</p>
      </aside>
    </div>
  </section>
</template>
