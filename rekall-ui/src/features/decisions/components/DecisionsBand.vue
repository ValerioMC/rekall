<script setup lang="ts">
import { computed } from 'vue'
import PlanTube from '@/components/ui/PlanTube.vue'
import SegmentedControl from '@/components/ui/SegmentedControl.vue'
import { describeTally, type DecisionCounts } from '@/features/decisions/lib/decision-tally'

export type DecisionsView = 'ledger' | 'read'

/** Where the decisions stand and that the tab follows the conversation; the person switches between the ledger and what a session reads. */
const props = defineProps<{ counts: DecisionCounts }>()
const view = defineModel<DecisionsView>({ required: true })

const tally = computed(() => describeTally(props.counts))
</script>

<template>
  <section class="flex flex-none flex-wrap items-center gap-x-5 gap-y-2 border-b border-border px-6 py-3" aria-label="Where the decisions stand" data-testid="decisions-band">
    <div class="flex min-w-[17.5rem] max-w-[28.75rem] flex-[0_1_28.75rem] flex-col gap-2">
      <PlanTube v-if="counts.total > 0" :done="counts.answered" :total="counts.total" size="lg" />
      <p class="text-small text-text-muted" data-testid="decisions-tally">
        <b class="font-semibold text-text">{{ tally.headline }}</b>
        {{ ' ' }}{{ tally.rest }}
      </p>
    </div>
    <span class="flex-1" />
    <p class="text-small text-text-muted whitespace-nowrap">Updates as you talk to Claude, in the terminal or the chat.</p>
    <SegmentedControl
      v-if="counts.total > 0"
      v-model="view"
      label="Show the decisions as"
      size="sm"
      :options="[
        { value: 'ledger', label: 'Ledger' },
        { value: 'read', label: 'As the session reads it' }
      ]"
    />
    <slot name="actions" />
  </section>
</template>
