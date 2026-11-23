<script setup lang="ts">
import { ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppField from '@/components/ui/AppField.vue'
import AppInput from '@/components/ui/AppInput.vue'
import { editProblem, failureText, startEditDraft, toEdit } from '@/features/decisions/lib/decision-draft'
import type { Decision } from '@/features/decisions/model/decision'
import { useConsoleStore } from '@/stores/console.store'

/**
 * The correction form, opened on request: a new answer, a note kept beside it, or both. The record
 * is saved as edited by the person, and the conversation excerpt stays as it was.
 */
const props = defineProps<{ decision: Decision }>()
const emit = defineEmits<{ saved: []; cancel: [] }>()

const store = useConsoleStore()
const draft = ref(startEditDraft(props.decision))
const problem = ref<string | null>(null)
const saving = ref(false)

async function save(): Promise<void> {
  problem.value = editProblem(props.decision, draft.value)
  if (problem.value !== null) return
  saving.value = true
  try {
    await store.editDecision(props.decision, toEdit(props.decision, draft.value))
    emit('saved')
  } catch (error) {
    problem.value = failureText(error)
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <form class="grid gap-3" aria-label="Correct this decision" data-testid="decision-edit-form" @submit.prevent="save">
    <AppField label="Answer" :error="problem" class="!mb-0">
      <template #default="{ fieldId, describedBy }">
        <textarea
          :id="fieldId"
          v-model="draft.answer"
          rows="2"
          class="field w-full resize-y rounded-control px-3 py-1.5 text-body text-text"
          :class="problem ? 'border-danger' : ''"
          :aria-describedby="describedBy"
          :aria-invalid="problem !== null"
          placeholder="What was decided"
          data-testid="decision-edit-answer"
        />
      </template>
    </AppField>
    <AppField label="Add a note for the session" hint="Optional. Dated and kept next to the answer." class="!mb-0">
      <template #default="{ fieldId, describedBy }">
        <AppInput :id="fieldId" v-model="draft.note" :described-by="describedBy" data-testid="decision-edit-note" />
      </template>
    </AppField>
    <div class="flex flex-wrap items-center gap-2">
      <AppButton type="submit" variant="primary" :loading="saving" data-testid="decision-edit-save">Save correction</AppButton>
      <AppButton variant="quiet" @click="emit('cancel')">Cancel</AppButton>
      <span class="text-caption text-text-subtle">Saved as edited by you. The conversation excerpt stays as it was.</span>
    </div>
  </form>
</template>
