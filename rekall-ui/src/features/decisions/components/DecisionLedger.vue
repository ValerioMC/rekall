<script setup lang="ts">
import { computed, ref, toRef } from 'vue'
import DecisionRow from '@/features/decisions/components/DecisionRow.vue'
import { useFreshDecisions } from '@/features/decisions/composables/useFreshDecisions'
import { groupByTopic } from '@/features/decisions/lib/decision-groups'
import type { Decision } from '@/features/decisions/model/decision'
import type { DecisionId, TaskId } from '@/model/branded'

/**
 * The decisions of a task grouped by topic, open questions first. A row stays open while the
 * record changes under it; a row that arrived or changed is marked for a moment and announced.
 */
const props = defineProps<{ taskId: TaskId; decisions: readonly Decision[]; hasSteps: boolean }>()

const opened = ref<DecisionId | null>(null)
const groups = computed(() => groupByTopic(props.decisions))
const { fresh, announcement } = useFreshDecisions(toRef(props, 'taskId'), toRef(props, 'decisions'))

function toggle(decision: Decision): void {
  opened.value = opened.value === decision.id ? null : decision.id
}
</script>

<template>
  <div class="min-h-0 flex-1 overflow-y-auto px-4 pb-6" data-testid="decision-ledger">
    <div
      class="sticky top-0 z-[2] grid grid-cols-[28px_minmax(0,5fr)_minmax(0,4fr)_minmax(0,11.5rem)] gap-x-3.5 border-b border-border bg-canvas px-2 py-2 text-caption font-medium text-text-subtle"
      aria-hidden="true"
    >
      <span />
      <span>Question</span>
      <span>Answer</span>
      <span class="text-right">Recorded from</span>
    </div>

    <section v-for="group in groups" :key="group.topic" :aria-label="group.topic" data-testid="decision-group">
      <div class="flex items-baseline gap-2.5 px-2 pb-1.5 pt-5">
        <h2 class="text-body font-semibold text-text">{{ group.topic }}</h2>
        <span class="font-mono text-caption text-text-subtle" data-testid="decision-group-count">{{ group.answered }} of {{ group.decisions.length }} answered</span>
        <span class="h-px flex-1 self-center bg-border" aria-hidden="true" />
      </div>
      <DecisionRow
        v-for="decision in group.decisions"
        :key="decision.id"
        :decision="decision"
        :has-steps="hasSteps"
        :opened="opened === decision.id"
        :fresh="fresh.has(decision.id)"
        @toggle="toggle(decision)"
      >
        <template v-if="$slots.actions" #actions>
          <slot name="actions" :decision="decision" />
        </template>
      </DecisionRow>
    </section>

    <p class="sr-only" role="status" aria-live="polite">{{ announcement }}</p>
  </div>
</template>
