<script setup lang="ts">
import { nextTick, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import DecisionEditForm from '@/features/decisions/components/DecisionEditForm.vue'
import { failureText } from '@/features/decisions/lib/decision-draft'
import type { Decision } from '@/features/decisions/model/decision'
import { useToastStore } from '@/stores/toast.store'
import { useConsoleStore } from '@/stores/console.store'

/** What a person can do to one decision by hand: correct it or remove it. A wrong record is best put right by telling Claude. */
const props = defineProps<{ decision: Decision }>()

const store = useConsoleStore()
const toast = useToastStore()
const editing = ref(false)
const confirming = ref(false)
const editButton = ref<InstanceType<typeof AppButton> | null>(null)

async function closeEditor(): Promise<void> {
  editing.value = false
  await nextTick()
  ;(editButton.value?.$el as HTMLElement | undefined)?.focus()
}

async function remove(): Promise<void> {
  confirming.value = false
  try {
    await store.removeDecision(props.decision)
  } catch (error) {
    toast.notify(failureText(error))
  }
}
</script>

<template>
  <div data-testid="decision-actions">
    <DecisionEditForm v-if="editing" :decision="decision" @saved="closeEditor" @cancel="closeEditor" />
    <div v-else class="flex flex-wrap items-center gap-1">
      <AppButton ref="editButton" variant="quiet" size="sm" data-testid="decision-edit" @click="editing = true">Edit</AppButton>
      <AppButton variant="danger-quiet" size="sm" data-testid="decision-remove" @click="confirming = true">Remove</AppButton>
      <span class="ml-auto text-caption text-text-subtle">Wrong? Tell Claude in the conversation and it corrects or removes this.</span>
    </div>
    <Transition name="dialog">
      <AppConfirm
        v-if="confirming"
        :title="`Remove ${decision.key}?`"
        body="Claude will no longer read this decision with the task."
        :blast="`Removes this decision from the task: ${decision.question}`"
        confirm-label="Remove decision"
        @cancel="confirming = false"
        @confirm="remove"
      />
    </Transition>
  </div>
</template>
