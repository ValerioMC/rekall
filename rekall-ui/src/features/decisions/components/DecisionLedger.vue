<script setup lang="ts">
import { computed, ref, toRef, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import DecisionFoundText from '@/features/decisions/components/DecisionFoundText.vue'
import DecisionRow from '@/features/decisions/components/DecisionRow.vue'
import { useFreshDecisions } from '@/features/decisions/composables/useFreshDecisions'
import { groupByTopic, type DecisionGroup } from '@/features/decisions/lib/decision-groups'
import type { DecisionFinds } from '@/features/decisions/lib/decision-search'
import type { Decision } from '@/features/decisions/model/decision'
import type { DecisionId, TaskId } from '@/model/branded'

/**
 * The decisions of a task grouped by topic, open questions first. A row stays open while the
 * record changes under it; a row that arrived or changed is marked for a moment and announced.
 * While a search is on (`finds` is not `null`) only the decisions it found are listed.
 */
const props = defineProps<{ taskId: TaskId; decisions: readonly Decision[]; hasSteps: boolean; query: string; finds: DecisionFinds | null }>()
const emit = defineEmits<{ clearSearch: [] }>()

const opened = ref<DecisionId | null>(null)
const scroller = ref<HTMLElement | null>(null)
const groups = computed(() =>
  groupByTopic(props.decisions)
    .map((group) => ({ ...group, listed: group.decisions.filter((decision) => props.finds?.has(decision.id) ?? true) }))
    .filter((group) => group.listed.length > 0)
)
const { fresh, announcement } = useFreshDecisions(toRef(props, 'taskId'), toRef(props, 'decisions'))

function toggle(decision: Decision): void {
  opened.value = opened.value === decision.id ? null : decision.id
}

function tallyOf(group: DecisionGroup & { listed: readonly Decision[] }): string {
  const total = group.decisions.length
  return props.finds === null ? `${group.answered} of ${total} answered` : `${group.listed.length} of ${total} found`
}

watch(
  () => props.query,
  () => {
    if (scroller.value) scroller.value.scrollTop = 0
  }
)
</script>

<template>
  <div ref="scroller" class="min-h-0 flex-1 overflow-y-auto px-4 pb-6" data-testid="decision-ledger">
    <div v-if="groups.length === 0" class="grid justify-items-center gap-3 px-6 py-18 text-center" data-testid="decision-none">
      <p class="max-w-[60ch] text-title font-semibold text-text wrap-anywhere">No decision contains “{{ query.trim() }}”</p>
      <p class="text-body text-text-subtle">Topics, questions, answers, notes, options, conversations and reasons were searched.</p>
      <AppButton size="sm" data-testid="decision-none-clear" @click="emit('clearSearch')">Clear search</AppButton>
    </div>

    <div
      v-else
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
        <h2 class="text-body font-semibold text-text"><DecisionFoundText :text="group.topic" :query="query" /></h2>
        <span class="font-mono text-caption text-text-subtle" data-testid="decision-group-count">{{ tallyOf(group) }}</span>
        <span class="h-px flex-1 self-center bg-border" aria-hidden="true" />
      </div>
      <DecisionRow
        v-for="decision in group.listed"
        :key="decision.id"
        :decision="decision"
        :has-steps="hasSteps"
        :opened="opened === decision.id"
        :fresh="fresh.has(decision.id)"
        :query="query"
        :find="finds?.get(decision.id)"
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
