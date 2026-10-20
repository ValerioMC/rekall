<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { readFlow } from '@/features/flow/lib/step-flow'
import type { Point } from '@/features/diagram/lib/viewport'
import type { RouteTarget } from '@/features/flow/lib/route-target'
import type { Task, TaskStep } from '@/features/catalog/model/catalog'
import type { StepLinkId, TaskStepId } from '@/model/branded'
import FlowCanvas from './FlowCanvas.vue'
import FlowInspector from './FlowInspector.vue'
import FlowTrail from './FlowTrail.vue'

/**
 * A task's steps as a flow: the trail of moves on top, the graph beside the picked step's
 * routes and visits. Everything is read from the steps, so a session's move redraws it live.
 */
const props = defineProps<{
  task: Task
  steps: readonly TaskStep[]
}>()

const emit = defineEmits<{ 'open-detail': [id: TaskStepId] }>()

const store = useConsoleStore()
const { run, isRunning } = useAsyncAction()

const flow = computed(() => readFlow(props.steps))
const selectedId = ref<TaskStepId | null>(null)
const selected = computed(() => flow.value.steps.find((step) => step.id === selectedId.value) ?? null)

// Follows the work: a new task, or a move, picks the step it stands on unless one is picked.
watch(
  () => [props.task.id, flow.value.currentId] as const,
  ([taskId], previous) => {
    const taskChanged = previous?.[0] !== taskId
    if (taskChanged || selected.value === null) selectedId.value = flow.value.currentId ?? flow.value.entryId
  },
  { immediate: true }
)

function addRoute(to: RouteTarget, outcome: string): void {
  if (!selectedId.value) return
  void run(() => store.addRoute(props.task.id, selectedId.value!, to, outcome))
}

function editRoute(
  id: StepLinkId,
  patch: { toStepId?: TaskStepId; endsFlow?: boolean; outcome?: string }
): void {
  void run(() => store.editRoute(props.task.id, id, patch))
}

function removeRoute(id: StepLinkId): void {
  void run(() => store.removeRoute(props.task.id, id))
}

function limitVisits(maxVisits: number | null): void {
  if (!selectedId.value) return
  void run(() => store.limitVisits(props.task.id, selectedId.value!, maxVisits))
}

// A route drawn on the canvas fills the picked step's route form and waits for its outcome.
const drawn = ref<{ to: RouteTarget; nonce: number } | null>(null)

function drawRoute(from: TaskStepId, to: RouteTarget): void {
  selectedId.value = from
  drawn.value = { to, nonce: (drawn.value?.nonce ?? 0) + 1 }
}

async function placeStep(id: TaskStepId, at: Point): Promise<void> {
  await run(() => store.placeStep(props.task.id, id, at))
}

function tidy(): void {
  void run(() => store.tidyFlow(props.task.id))
}

const confirmingReset = ref(false)

async function reset(): Promise<void> {
  confirmingReset.value = false
  await run(() => store.resetFlow(props.task.id))
}
</script>

<template>
  <div class="@container flex min-h-0 min-w-0 flex-1 flex-col" data-testid="step-flow-view">
    <FlowTrail
      :trail="flow.trail"
      :selected-id="selectedId"
      @select="(id) => (selectedId = id)"
      @reset="confirmingReset = true"
    />

    <p
      v-if="!flow.isFlow"
      class="shrink-0 border-b border-border bg-anchor-soft px-5 py-2 text-caption text-anchor-strong"
      data-testid="flow-hint"
    >
      These steps are still a list. Pick a step and add a route to say where each outcome sends the work.
    </p>

    <div class="flex min-h-0 flex-1 flex-col @3xl:flex-row">
      <FlowCanvas
        :flow="flow"
        :selected-id="selectedId"
        @select="(id) => (selectedId = id)"
        @place="placeStep"
        @connect="drawRoute"
        @tidy="tidy"
      />
      <FlowInspector
        v-if="selected"
        class="max-h-[48%] shrink-0 border-t border-border bg-surface @3xl:max-h-none @3xl:w-[340px] @3xl:border-t-0 @3xl:border-l"
        :step="selected"
        :steps="flow.steps"
        :current="selected.id === flow.currentId"
        :busy="isRunning"
        :drawn="drawn"
        @select="(id) => (selectedId = id)"
        @open-detail="emit('open-detail', selected.id)"
        @add-route="addRoute"
        @edit-route="editRoute"
        @remove-route="removeRoute"
        @limit-visits="limitVisits"
      />
    </div>

    <Transition name="dialog">
      <AppConfirm
        v-if="confirmingReset"
        title="Reset the moves?"
        body="Forgets the trail and every visit count on this task. The steps keep their states and routes."
        :blast="`${flow.trail.length} moves · not recoverable`"
        confirm-label="Reset moves"
        @cancel="confirmingReset = false"
        @confirm="reset"
      />
    </Transition>
  </div>
</template>
