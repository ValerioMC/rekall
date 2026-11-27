<script setup lang="ts">
import { computed } from 'vue'
import PlanTube from '@/components/ui/PlanTube.vue'
import { describeTally, type DecisionCounts } from '@/features/decisions/lib/decision-tally'

/** Where the decisions stand and that the tab follows the conversation, beside the tab's actions. */
const props = defineProps<{ counts: DecisionCounts }>()

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
    <p class="text-small text-text-muted whitespace-nowrap">Updates as you talk in your session.</p>
    <slot name="actions" />
  </section>
</template>
