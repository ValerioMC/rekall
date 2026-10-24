<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { ROUTES_PER_STEP_MAX, type DraftRoute, type DraftStep } from '@/features/graph/model/deployment-graph'

/**
 * The routes out of one step: when a session claims it with this outcome, the work moves there.
 * A step with no route ends the flow. The end and every other step are the targets on offer.
 */
const props = defineProps<{
  step: DraftStep
  steps: readonly DraftStep[]
  /** The route just drawn on the canvas, whose outcome is focused to be named. */
  focusIndex: number | null
}>()

const emit = defineEmits<{
  add: []
  update: [index: number, patch: Partial<DraftRoute>]
  remove: [index: number]
}>()

const END = ''

const targets = computed(() => [
  ...props.steps
    .map((step, position) => ({ value: step.key, label: `${position + 1}. ${step.title || 'Untitled step'}` }))
    .filter((target) => target.value !== props.step.key),
  { value: END, label: 'End of the flow' }
])

const outcomeInputs = ref<HTMLInputElement[]>([])

watch(
  () => props.focusIndex,
  async (index) => {
    if (index === null) return
    await nextTick()
    outcomeInputs.value[index]?.select()
  }
)

function retarget(index: number, value: string): void {
  emit('update', index, { toKey: value === END ? null : value })
}

function repeated(route: DraftRoute, index: number): boolean {
  return props.step.routes.some((other, at) => at !== index && other.outcome.trim() === route.outcome.trim())
}
</script>

<template>
  <div class="flex flex-col gap-1.5" data-testid="graph-routes">
    <p v-if="!step.routes.length" class="text-caption leading-relaxed text-text-subtle">
      No route: claiming this step ends the flow.
    </p>
    <div
      v-for="(route, index) in step.routes"
      :key="index"
      class="grid grid-cols-[minmax(0,0.9fr)_auto_minmax(0,1.1fr)_auto] items-center gap-1.5"
      data-testid="graph-route-row"
    >
      <label class="sr-only" :for="`route-outcome-${step.key}-${index}`">Outcome</label>
      <input
        :id="`route-outcome-${step.key}-${index}`"
        ref="outcomeInputs"
        :value="route.outcome"
        maxlength="60"
        autocomplete="off"
        spellcheck="false"
        placeholder="outcome"
        class="field h-8 w-full rounded-control px-2 font-mono text-caption text-text"
        :class="repeated(route, index) && 'border-danger'"
        :aria-invalid="repeated(route, index) || undefined"
        data-testid="graph-route-outcome"
        @input="emit('update', index, { outcome: ($event.target as HTMLInputElement).value })"
      />
      <span class="text-text-subtle" aria-hidden="true">&rarr;</span>
      <label class="sr-only" :for="`route-target-${step.key}-${index}`">Leads to</label>
      <div class="relative">
        <select
          :id="`route-target-${step.key}-${index}`"
          :value="route.toKey ?? END"
          class="field h-8 w-full cursor-pointer appearance-none truncate rounded-control pl-2 pr-6 text-caption text-text"
          data-testid="graph-route-target"
          @change="retarget(index, ($event.target as HTMLSelectElement).value)"
        >
          <option v-for="target in targets" :key="target.value" :value="target.value">{{ target.label }}</option>
        </select>
        <svg
          class="pointer-events-none absolute right-2 top-1/2 size-3 -translate-y-1/2 text-text-subtle"
          viewBox="0 0 24 24"
          fill="none"
          aria-hidden="true"
        >
          <path d="M6 9l6 6 6-6" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
        </svg>
      </div>
      <button
        type="button"
        class="focus-ring grid size-8 cursor-pointer place-items-center rounded-control text-text-subtle transition-colors hover:bg-danger-soft hover:text-danger"
        :aria-label="`Remove the route ${route.outcome}`"
        data-testid="graph-route-remove"
        @click="emit('remove', index)"
      >
        <svg class="size-3.5" viewBox="0 0 14 14" fill="none" aria-hidden="true">
          <path d="M3.5 3.5l7 7M10.5 3.5l-7 7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
      </button>
    </div>
    <button
      v-if="step.routes.length < ROUTES_PER_STEP_MAX"
      type="button"
      class="key-quiet focus-ring mt-0.5 inline-flex h-8 cursor-pointer items-center gap-1.5 self-start rounded-control px-2.5 text-caption"
      data-testid="graph-route-add"
      @click="emit('add')"
    >
      <svg class="size-3" viewBox="0 0 14 14" fill="none" aria-hidden="true">
        <path d="M7 2.5v9M2.5 7h9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
      </svg>
      Add route
    </button>
  </div>
</template>
