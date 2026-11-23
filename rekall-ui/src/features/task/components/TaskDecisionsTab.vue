<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import DecisionAddForm from '@/features/decisions/components/DecisionAddForm.vue'
import DecisionLedger from '@/features/decisions/components/DecisionLedger.vue'
import DecisionRowActions from '@/features/decisions/components/DecisionRowActions.vue'
import DecisionSessionView from '@/features/decisions/components/DecisionSessionView.vue'
import DecisionsBand, { type DecisionsView } from '@/features/decisions/components/DecisionsBand.vue'
import { countDecisions } from '@/features/decisions/lib/decision-tally'
import { NO_DECISIONS } from '@/features/decisions/model/decision'
import { useConsoleStore } from '@/stores/console.store'
import type { Task } from '@/features/catalog/model/catalog'

/**
 * The Decisions tab: what was settled about the task before it runs, recorded by Claude as the
 * person goes through it in the terminal or the chat. It follows the conversation by itself.
 * It runs parallel to the plan: it adds no count to the header, the tabs or the review queue.
 */
const props = defineProps<{ task: Task }>()

const store = useConsoleStore()
const { decisionsByTask, steps } = storeToRefs(store)
const view = ref<DecisionsView>('ledger')
const adding = ref(false)

const shown = computed(() => decisionsByTask.value[props.task.id] ?? NO_DECISIONS)
const counts = computed(() => countDecisions(shown.value.decisions))
const hasSteps = computed(() => steps.value.some((step) => step.taskId === props.task.id && step.state !== 'DRAFT'))
const stepChoices = computed(() =>
  steps.value
    .filter((step) => step.taskId === props.task.id && step.state !== 'DRAFT')
    .sort((a, b) => a.position - b.position)
    .map((step, index) => ({ number: index + 1, title: step.title }))
)
const showsBlock = computed(() => view.value === 'read' && counts.value.total > 0)

watch(
  () => props.task.id,
  (id) => void store.loadDecisions(id).catch(() => undefined),
  { immediate: true }
)
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col" data-testid="decisions-tab">
    <DecisionsBand v-model="view" :counts="counts">
      <template #actions>
        <AppButton size="sm" :disabled="adding" data-testid="decision-add" @click="adding = true">Add decision</AppButton>
      </template>
    </DecisionsBand>
    <DecisionAddForm v-if="adding" :task-id="task.id" :steps="stepChoices" @added="adding = false" @cancel="adding = false" />
    <DecisionSessionView v-if="showsBlock" :block="shown.sessionBlock" :task-code="task.code" />
    <DecisionLedger v-else-if="counts.total > 0" :task-id="task.id" :decisions="shown.decisions" :has-steps="hasSteps">
      <template #actions="{ decision }"><DecisionRowActions :decision="decision" /></template>
    </DecisionLedger>
    <div v-else-if="!adding" class="grid flex-1 place-items-center p-10">
      <AppEmptyState
        title="Nothing recorded yet"
        description="When you go through the task with Claude in the terminal or the chat, to check what is missing and what could be better, Claude records each question and your answer here. There is nothing to do in this tab."
      >
        <p class="max-w-[56ch] text-small text-text-muted">
          Start with <code class="rounded-key bg-surface-raised px-1.5 py-px font-mono text-small text-text-muted">/rk task:{{ task.code }} refine</code>.
        </p>
      </AppEmptyState>
    </div>
  </div>
</template>
