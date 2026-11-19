<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'
import { useDismissable } from '@/composables/useDismissable'
import type { FlowMove } from '@/features/flow/lib/step-flow'
import type { TaskStepId } from '@/model/branded'

/**
 * The way the work went, move by move, behind a count on the canvas: each step it entered with its
 * move number and the outcome it left by, the open one last. An outcome left with a report is
 * underlined and shows it on hover. Reset forgets the trail.
 */
defineProps<{
  trail: readonly FlowMove[]
  selectedId: TaskStepId | null
}>()

const emit = defineEmits<{
  select: [id: TaskStepId]
  reset: []
}>()

const open = ref(false)
const host = ref<HTMLElement | null>(null)
const list = ref<HTMLElement | null>(null)
useDismissable(host, open)

function moveTitle(move: FlowMove): string {
  const entered = `Move ${move.ordinal}: ${move.step.title}`
  return move.report ? `${entered}, left by ${move.outcome ?? 'its only route'}: ${move.report}` : entered
}

function pick(id: TaskStepId): void {
  emit('select', id)
  open.value = false
}

function reset(): void {
  open.value = false
  emit('reset')
}

watch(open, async (isOpen) => {
  if (!isOpen) return
  await nextTick()
  if (list.value) list.value.scrollTop = list.value.scrollHeight
})
</script>

<template>
  <div ref="host" data-testid="flow-moves">
    <button
      type="button"
      class="focus-ring flex h-8 cursor-pointer items-center gap-1.5 rounded-card border border-border bg-surface px-3 text-small text-text-muted shadow-raised transition-colors hover:text-text"
      :aria-expanded="open"
      aria-controls="flow-moves-list"
      data-testid="flow-move-count"
      @click="open = !open"
    >
      <b class="font-mono font-medium tabular-nums text-accent">{{ trail.length }}</b>
      {{ trail.length === 1 ? 'move' : 'moves' }}
      <svg class="size-2.5 text-text-subtle transition-transform" :class="open && 'rotate-90'" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <path d="M9 6l6 6-6 6" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </button>

    <section
      v-if="open"
      id="flow-moves-list"
      class="absolute left-0 top-full z-(--z-overlay) mt-1.5 w-[340px] rounded-card border border-border-strong bg-surface p-3 shadow-(--shadow-lift)"
      aria-label="Moves through the flow, oldest first"
      data-testid="flow-trail"
    >
      <ol v-if="trail.length" ref="list" class="grid max-h-72 gap-px overflow-y-auto">
        <li v-for="move in trail" :key="move.ordinal">
          <button
            type="button"
            class="focus-ring grid w-full cursor-pointer grid-cols-[1.75rem_minmax(0,1fr)_auto] items-baseline gap-2 rounded-key px-1.5 py-1 text-left text-caption hover:bg-surface-raised"
            :class="[move.open && 'text-accent-strong', move.step.id === selectedId && 'bg-surface-raised']"
            :aria-current="move.open ? 'step' : undefined"
            :title="moveTitle(move)"
            data-testid="flow-move"
            @click="pick(move.step.id)"
          >
            <span class="text-right font-mono tabular-nums text-text-subtle">{{ move.ordinal }}</span>
            <span class="truncate" :class="!move.open && 'text-text'">{{ move.step.title }}</span>
            <span
              v-if="move.outcome"
              class="font-mono text-micro text-text-subtle"
              :class="move.report && 'cursor-help underline decoration-dotted underline-offset-2'"
              :title="move.report ?? undefined"
              data-testid="flow-move-outcome"
              >{{ move.outcome }} ›</span
            >
            <span v-else-if="!move.open" class="font-mono text-micro text-text-subtle" aria-hidden="true">›</span>
          </button>
        </li>
      </ol>
      <p v-else class="text-caption text-text-subtle">Not started. The first step a session starts is move 1.</p>

      <div v-if="trail.length" class="mt-2 flex justify-end border-t border-border pt-2">
        <button
          type="button"
          class="key-quiet focus-ring cursor-pointer rounded-key px-2.5 py-1 text-caption"
          data-testid="flow-reset"
          @click="reset"
        >
          Reset moves
        </button>
      </div>
    </section>
  </div>
</template>
