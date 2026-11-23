<script setup lang="ts">
import { computed, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppField from '@/components/ui/AppField.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppSelect, { type SelectOption } from '@/components/ui/AppSelect.vue'
import { addProblem, EMPTY_ADD_DRAFT, type AddProblem, failureText, toNewDecision } from '@/features/decisions/lib/decision-draft'
import { useConsoleStore } from '@/stores/console.store'
import type { TaskId } from '@/model/branded'

export interface StepChoice {
  readonly number: number
  readonly title: string
}

/** A decision the person writes by hand: answered from the start, marked as edited by them, with no conversation excerpt. */
const props = defineProps<{ taskId: TaskId; steps: readonly StepChoice[] }>()
const emit = defineEmits<{ added: []; cancel: [] }>()

const store = useConsoleStore()
const draft = ref({ ...EMPTY_ADD_DRAFT })
const problem = ref<AddProblem | null>(null)
const saving = ref(false)
const stepOptions = computed<readonly SelectOption[]>(() => [
  { value: '', label: 'The task as a whole' },
  ...props.steps.map((step) => ({ value: String(step.number), label: `Step ${step.number} · ${step.title}` }))
])

async function add(): Promise<void> {
  problem.value = addProblem(draft.value)
  if (problem.value !== null) return
  saving.value = true
  try {
    await store.addDecision(props.taskId, toNewDecision(draft.value))
    emit('added')
  } catch (error) {
    problem.value = { field: 'form', message: failureText(error) }
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <form class="mx-2 mt-3.5 grid max-w-[47.5rem] gap-3 rounded-card border border-border-strong bg-surface px-4 py-3.5" aria-label="Add a decision" data-testid="decision-add-form" @submit.prevent="add">
    <h2 class="text-title font-semibold text-text">Add a decision</h2>
    <AppField label="Topic" hint="Groups it in the ledger. Optional." class="!mb-0">
      <template #default="{ fieldId, describedBy }">
        <AppInput :id="fieldId" v-model="draft.topic" :described-by="describedBy" placeholder="Secrets, Database, Ingress" data-testid="decision-add-topic" />
      </template>
    </AppField>
    <AppField label="Question" :error="problem?.field === 'question' ? problem.message : null" class="!mb-0">
      <template #default="{ fieldId, describedBy }">
        <AppInput :id="fieldId" v-model="draft.question" :described-by="describedBy" placeholder="What was decided about?" data-testid="decision-add-question" />
      </template>
    </AppField>
    <AppField label="Answer" :error="problem?.field === 'answer' ? problem.message : null" class="!mb-0">
      <template #default="{ fieldId, describedBy }">
        <textarea
          :id="fieldId"
          v-model="draft.answer"
          rows="2"
          class="field w-full resize-y rounded-control px-3 py-1.5 text-body text-text"
          :class="problem?.field === 'answer' ? 'border-danger' : ''"
          :aria-describedby="describedBy"
          :aria-invalid="problem?.field === 'answer'"
          placeholder="What was decided"
          data-testid="decision-add-answer"
        />
      </template>
    </AppField>
    <AppField v-if="steps.length > 0" label="Read at" hint="A step's decisions are read when that step runs." class="!mb-0">
      <template #default="{ fieldId, describedBy }">
        <AppSelect :id="fieldId" v-model="draft.step" :options="stepOptions" :described-by="describedBy" data-testid="decision-add-step" />
      </template>
    </AppField>
    <p v-if="problem?.field === 'form'" class="text-caption text-danger" role="alert">{{ problem.message }}</p>
    <div class="flex flex-wrap items-center gap-2">
      <AppButton type="submit" variant="primary" :loading="saving" data-testid="decision-add-save">Add to the record</AppButton>
      <AppButton variant="quiet" @click="emit('cancel')">Cancel</AppButton>
      <span class="text-caption text-text-subtle">Marked edited by you, with no conversation excerpt.</span>
    </div>
  </form>
</template>
