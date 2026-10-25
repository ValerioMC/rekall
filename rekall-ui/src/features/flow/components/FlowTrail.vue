<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'
import type { FlowMove } from '@/features/flow/lib/step-flow'
import type { TaskStepId } from '@/model/branded'

/**
 * The way the work went, move by move: each step it entered with its move number and the
 * outcome it left by, the open one last. An outcome left with a report is underlined and shows
 * it on hover. The move count leads; reset forgets the trail.
 */
const props = defineProps<{
  trail: readonly FlowMove[]
  selectedId: TaskStepId | null
}>()

defineEmits<{
  select: [id: TaskStepId]
  reset: []
}>()

const strip = ref<HTMLElement | null>(null)

function moveTitle(move: FlowMove): string {
  const entered = `Move ${move.ordinal}: ${move.step.title}`
  return move.report ? `${entered}, left by ${move.outcome ?? 'its only route'}: ${move.report}` : entered
}

// A new move lands at the end of the strip; keep it in view.
watch(
  () => props.trail.length,
  async () => {
    await nextTick()
    strip.value?.scrollTo?.({ left: strip.value.scrollWidth })
  }
)
</script>

<template>
  <div class="flex shrink-0 items-center gap-3 border-b border-border px-5 py-2.5" data-testid="flow-trail">
    <p
      class="shrink-0 font-mono text-small font-semibold leading-none tabular-nums"
      data-testid="flow-move-count"
    >
      <span class="text-accent">{{ trail.length }}</span>
      <span class="ml-1 font-sans text-caption font-normal text-text-subtle">{{
        trail.length === 1 ? 'move' : 'moves'
      }}</span>
    </p>

    <ol
      v-if="trail.length"
      ref="strip"
      class="flex min-w-0 flex-1 items-center gap-1 overflow-x-auto py-0.5"
      aria-label="Moves through the flow, oldest first"
    >
      <li v-for="move in trail" :key="move.ordinal" class="flex shrink-0 items-center gap-1">
        <button
          type="button"
          class="focus-ring inline-flex cursor-pointer items-center gap-1.5 rounded-full border px-2 py-0.5 text-caption transition-colors"
          :class="[
            move.open
              ? 'border-accent bg-accent-soft text-accent-strong'
              : 'border-border text-text-muted hover:border-border-strong hover:text-text',
            move.step.id === selectedId && 'outline outline-1 outline-anchor'
          ]"
          :aria-current="move.open ? 'step' : undefined"
          :title="moveTitle(move)"
          data-testid="flow-move"
          @click="$emit('select', move.step.id)"
        >
          <span class="font-mono text-micro tabular-nums opacity-70">{{ move.ordinal }}</span>
          <span class="max-w-[14ch] truncate">{{ move.step.title }}</span>
        </button>
        <span
          v-if="move.outcome"
          class="font-mono text-micro text-text-subtle"
          :class="move.report && 'cursor-help underline decoration-dotted underline-offset-2'"
          :title="move.report ?? undefined"
          aria-hidden="true"
          data-testid="flow-move-outcome"
          >{{ move.outcome }} ›</span
        >
        <span v-else-if="!move.open" class="text-micro text-text-subtle" aria-hidden="true">›</span>
      </li>
    </ol>
    <p v-else class="min-w-0 flex-1 truncate text-caption text-text-subtle">
      Not started. The first step a session starts is move 1.
    </p>

    <button
      v-if="trail.length"
      type="button"
      class="key-quiet focus-ring shrink-0 cursor-pointer rounded-control px-2.5 py-1 text-caption"
      data-testid="flow-reset"
      @click="$emit('reset')"
    >
      Reset moves
    </button>
  </div>
</template>
