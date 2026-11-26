<script setup lang="ts">
import AppButton from '@/components/ui/AppButton.vue'
import OpenTerminalButton from '@/features/claude/components/OpenTerminalButton.vue'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskStep } from '@/features/steps/model/task-step'

/**
 * The keys a step's state offers a person, kept in the dock so they sit where reading ends:
 * accept or send back a claim, tick or hand an open step to a session. A done step has no key
 * here: reopening it is in the header. Only a person marks a step done.
 */
defineProps<{ step: TaskStep; task: Task }>()

defineEmits<{ accept: []; 'send-back': []; 'mark-done': [] }>()
</script>

<template>
  <div class="flex items-center gap-2" data-testid="decision-keys">
    <template v-if="step.state === 'CLAIMED'">
      <AppButton variant="secondary" data-testid="step-send-back" @click="$emit('send-back')">Send back</AppButton>
      <AppButton variant="primary" data-testid="step-accept" @click="$emit('accept')">Accept</AppButton>
    </template>

    <template v-else-if="step.state === 'OPEN' || step.state === 'RUNNING'">
      <OpenTerminalButton :task-id="task.id" :step-id="step.id" :folder="task.projectRepoFolder" />
      <AppButton data-testid="decision-mark-done" @click="$emit('mark-done')">Mark done myself</AppButton>
    </template>
  </div>
</template>
