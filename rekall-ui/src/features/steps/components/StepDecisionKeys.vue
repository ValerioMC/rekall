<script setup lang="ts">
import AppButton from '@/components/ui/AppButton.vue'
import type { TaskStep } from '@/features/steps/model/task-step'

/**
 * The keys a step's state offers a person, docked where reading ends: send back or accept a claim,
 * or tick an open or running step by hand. A done step has none: reopening it is in the header.
 * Only a person marks a step done.
 */
defineProps<{ step: TaskStep }>()

defineEmits<{ accept: []; 'send-back': []; 'mark-done': [] }>()
</script>

<template>
  <div class="flex items-center gap-2" data-testid="decision-keys">
    <template v-if="step.state === 'CLAIMED'">
      <AppButton variant="secondary" data-testid="step-send-back" @click="$emit('send-back')">Send back</AppButton>
      <AppButton variant="primary" data-testid="step-accept" @click="$emit('accept')">Accept</AppButton>
    </template>

    <AppButton v-else-if="step.state === 'OPEN' || step.state === 'RUNNING'" data-testid="decision-mark-done" @click="$emit('mark-done')">
      Mark done myself
    </AppButton>
  </div>
</template>
