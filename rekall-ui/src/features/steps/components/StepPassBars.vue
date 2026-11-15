<script setup lang="ts">
import { computed } from 'vue'
import { passesOf, passesSummary } from '@/features/adherence/lib/adherence-figures'
import type { TaskStep } from '@/features/steps/model/task-step'

/**
 * One short bar for each pass at a step: yellow for one sent back, green for the one accepted,
 * indigo for the one a session has in hand. The words are the label, for those who cannot tell
 * the colours apart.
 */
const props = defineProps<{ step: TaskStep }>()

const passes = computed(() => passesOf(props.step))

const COLOUR = { 'sent-back': 'bg-warn', accepted: 'bg-safe', 'in-hand': 'bg-session' } as const
</script>

<template>
  <span v-if="passes.length" class="flex shrink-0 gap-0.5" role="img" :aria-label="passesSummary(step)" data-testid="step-passes">
    <span
      v-for="(kind, index) in passes"
      :key="index"
      class="h-1.5 w-3.5 rounded-key"
      :class="COLOUR[kind]"
      data-testid="step-pass-bar"
      :data-kind="kind"
    />
  </span>
</template>
