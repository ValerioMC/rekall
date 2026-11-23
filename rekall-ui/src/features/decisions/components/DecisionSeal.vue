<script setup lang="ts">
import { STATE_LABEL } from '@/features/decisions/lib/decision-labels'
import type { DecisionState } from '@/features/decisions/model/decision'

/** The state of a decision as a drawn mark: a question, a check or a pause, never colour alone. */
defineProps<{ state: DecisionState; opened?: boolean }>()

const TONE: Readonly<Record<DecisionState, string>> = {
  OPEN: 'border-border-field text-text-muted',
  ANSWERED: 'border-safe bg-safe text-canvas',
  ON_HOLD: 'border-warn bg-warn-soft text-warn'
}
</script>

<template>
  <span
    class="mt-px grid size-[22px] shrink-0 place-items-center rounded-full border-[1.5px] transition-colors duration-(--motion-move)"
    :class="state === 'OPEN' && opened ? 'border-accent bg-accent-soft text-accent' : TONE[state]"
    role="img"
    :aria-label="STATE_LABEL[state]"
    data-testid="decision-seal"
  >
    <svg viewBox="0 0 16 16" class="size-3" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
      <template v-if="state === 'OPEN'">
        <path stroke-width="1.6" d="M6 6.1a2.1 2.1 0 1 1 3.5 1.6c-.9.6-1.5 1.1-1.5 2.1" />
        <path stroke-width="1.6" d="M8 12.4v.1" />
      </template>
      <path v-else-if="state === 'ANSWERED'" stroke-width="1.8" d="M3.2 8.4 6.6 11.6 12.8 4.8" />
      <path v-else stroke-width="1.8" d="M6 4.5v7M10 4.5v7" />
    </svg>
  </span>
</template>
