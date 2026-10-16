<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import AppSelect, { type SelectOption } from '@/components/ui/AppSelect.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import { FLOW_END, isFlowEnd, type RouteTarget } from '@/features/flow/lib/route-target'
import type { StepLink, TaskStep } from '@/features/catalog/model/catalog'
import type { StepLinkId, TaskStepId } from '@/model/branded'

/**
 * The routes leaving one step: what it can end with and where each outcome sends the work.
 * An outcome is renamed on blur or Enter; the target changes as soon as it is picked.
 */
const props = defineProps<{
  step: TaskStep
  steps: readonly TaskStep[]
  busy: boolean
  /** A route drawn on the canvas: fills the target and asks for its outcome. */
  drawn?: { to: RouteTarget; nonce: number } | null
}>()

const emit = defineEmits<{
  add: [to: RouteTarget, outcome: string]
  edit: [id: StepLinkId, patch: { toStepId?: TaskStepId; endsFlow?: boolean; outcome?: string }]
  remove: [id: StepLinkId]
}>()

const END_LABEL = 'End of the flow'

const targets = computed<SelectOption[]>(() => [
  { value: FLOW_END, label: END_LABEL },
  ...props.steps
    .filter((candidate) => candidate.id !== props.step.id)
    .map((candidate) => ({ value: candidate.id, label: `${candidate.position + 1}. ${candidate.title}` }))
])

function targetName(link: StepLink): string {
  if (link.toStepId === null) return END_LABEL.toLowerCase()
  return props.steps.find((candidate) => candidate.id === link.toStepId)?.title ?? 'its step'
}

function retarget(link: StepLink, value: string): void {
  const target = value as RouteTarget
  emit('edit', link.id, isFlowEnd(target) ? { endsFlow: true } : { toStepId: target })
}

const outcomeField = ref<HTMLInputElement | null>(null)

const newOutcome = ref('')
const newTarget = ref<string | null>(null)
const canAdd = computed(
  () => newOutcome.value.trim() !== '' && newTarget.value !== null && newTarget.value !== ''
)

watch(
  () => props.step.id,
  () => {
    newOutcome.value = ''
    newTarget.value = null
  }
)

watch(
  () => props.drawn?.nonce,
  async () => {
    if (!props.drawn) return
    newTarget.value = props.drawn.to
    await nextTick()
    outcomeField.value?.focus()
  }
)

function add(): void {
  if (!canAdd.value) return
  emit('add', newTarget.value as RouteTarget, newOutcome.value.trim())
  newOutcome.value = ''
  newTarget.value = null
}

function renameOutcome(link: StepLink, event: Event): void {
  const typed = (event.target as HTMLInputElement).value.trim()
  if (typed === '' || typed === link.outcome) {
    ;(event.target as HTMLInputElement).value = link.outcome
    return
  }
  emit('edit', link.id, { outcome: typed })
}
</script>

<template>
  <section aria-labelledby="flow-routes-out" data-testid="flow-route-editor">
    <h4 id="flow-routes-out" class="eyebrow mb-2">When it ends</h4>

    <ul v-if="step.links.length" class="mb-3 space-y-1.5">
      <li
        v-for="link in step.links"
        :key="link.id"
        class="flex items-center gap-1.5"
        data-testid="flow-route-row"
      >
        <input
          :value="link.outcome"
          class="field h-8 w-[38%] min-w-0 rounded-control px-2 font-mono text-caption text-text"
          :aria-label="`Outcome of the route to ${targetName(link)}`"
          :disabled="busy"
          @keydown.enter.prevent="($event.target as HTMLInputElement).blur()"
          @blur="renameOutcome(link, $event)"
        />
        <span class="shrink-0 text-text-subtle" aria-hidden="true">→</span>
        <div class="min-w-0 flex-1">
          <AppSelect
            :model-value="link.toStepId ?? FLOW_END"
            :options="targets"
            :disabled="busy"
            @update:model-value="(value) => retarget(link, value as string)"
          />
        </div>
        <button
          type="button"
          class="focus-ring grid size-8 shrink-0 cursor-pointer place-items-center rounded-control text-text-subtle transition-colors hover:bg-danger-soft hover:text-danger"
          :aria-label="`Remove the route ${link.outcome}`"
          :disabled="busy"
          @click="emit('remove', link.id)"
        >
          <CloseGlyph />
        </button>
      </li>
    </ul>
    <p v-else class="mb-3 text-caption text-text-subtle">
      No routes out. Reaching this step ends the flow at End.
    </p>

    <form class="flex items-center gap-1.5" data-testid="flow-route-add" @submit.prevent="add">
      <input
        ref="outcomeField"
        v-model="newOutcome"
        class="field h-8 w-[38%] min-w-0 rounded-control px-2 font-mono text-caption text-text"
        placeholder="outcome"
        aria-label="Outcome of the new route, for example app-error"
        :disabled="busy || !targets.length"
      />
      <span class="shrink-0 text-text-subtle" aria-hidden="true">→</span>
      <div class="min-w-0 flex-1">
        <AppSelect
          v-model="newTarget"
          :options="targets"
          placeholder="Go to step…"
          :disabled="busy || !targets.length"
        />
      </div>
      <button
        type="submit"
        class="key-gold focus-ring h-8 shrink-0 cursor-pointer rounded-control px-3 text-caption font-semibold disabled:cursor-not-allowed disabled:opacity-50"
        :disabled="busy || !canAdd"
      >
        Add
      </button>
    </form>
    <p class="mt-2 text-micro leading-relaxed text-text-subtle">
      A session claims this step with the outcome that happened and the work moves to its step, or finishes at
      End. One route needs no outcome.
    </p>
  </section>
</template>
