<script setup lang="ts">
import { computed } from 'vue'
import { contextPercent, describeContext } from '@/features/chat/lib/context-readout'
import { formatCost } from '@/features/chat/lib/format-cost'
import type { TurnContext } from '@/features/chat/model/chat-event'

/**
 * What the live session has used so far: the share of the context window held after its last turn
 * and what it has cost in all. Nothing is shown until a turn has reported either.
 */
const props = defineProps<{ context: TurnContext | null; cost: number }>()

const shown = computed(() => props.context !== null || props.cost > 0)
</script>

<template>
  <p v-if="shown" class="flex items-center gap-2 font-mono text-micro text-text-subtle tabular-nums" data-testid="chat-readout">
    <span v-if="context" :title="describeContext(context)" data-testid="chat-readout-context">context {{ contextPercent(context) }}%</span>
    <span v-if="cost > 0" title="Cost of the session so far" data-testid="chat-readout-cost">{{ formatCost(cost) }}</span>
  </p>
</template>
