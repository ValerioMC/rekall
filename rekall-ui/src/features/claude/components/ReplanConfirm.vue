<script setup lang="ts">
import AppConfirm from '@/components/ui/AppConfirm.vue'

const MISSED_MESSAGE_MAX_CHARACTERS = 2000

/** The question before a replan deletes the draft and open steps; it also takes what the plan missed. */
defineProps<{ replaceableSteps: number }>()

const missedMessage = defineModel<string>('message', { required: true })

defineEmits<{ cancel: []; confirm: [] }>()

function blast(count: number): string {
  return `${count} ${count === 1 ? 'step' : 'steps'} deleted · not kept anywhere · the new plan is ready to work`
}
</script>

<template>
  <Transition name="dialog">
    <AppConfirm
      title="Replan this task?"
      body="A session redesigns the steps, taking in what you add. Its draft and open steps are replaced; running, claimed and done steps stay."
      :blast="blast(replaceableSteps)"
      confirm-label="Replan"
      @cancel="$emit('cancel')"
      @confirm="$emit('confirm')"
    >
      <label class="mt-3.5 block">
        <span class="eyebrow mb-1.5 block text-caption">
          What did the plan miss?
          <span class="font-normal normal-case tracking-normal text-text-subtle">optional</span>
        </span>
        <textarea
          v-model="missedMessage"
          rows="3"
          :maxlength="MISSED_MESSAGE_MAX_CHARACTERS"
          class="field w-full resize-y rounded-control px-3 py-2.5 text-body leading-relaxed text-text"
          placeholder="Something to add, or a step that should change"
          data-testid="replan-message"
        />
      </label>
    </AppConfirm>
  </Transition>
</template>
