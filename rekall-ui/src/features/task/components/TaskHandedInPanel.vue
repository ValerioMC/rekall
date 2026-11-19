<script setup lang="ts">
import AppButton from '@/components/ui/AppButton.vue'
import StepSeal from '@/features/steps/components/StepSeal.vue'
import { useTaskReviewActions } from '@/features/task/composables/useTaskReviewActions'
import type { Task } from '@/features/catalog/model/catalog'

/** The decision on a task a session handed in whole, because it has no checklist to claim step by step. */
const props = defineProps<{ task: Task }>()

const review = useTaskReviewActions(() => props.task)
</script>

<template>
  <section class="flex flex-col gap-4" aria-label="Decision" data-testid="handed-in-panel">
    <header>
      <p class="flex items-center gap-2 text-caption text-text-subtle">
        <span class="size-5.5" aria-hidden="true"><StepSeal state="CLAIMED" /></span>
        Handed in whole
      </p>
      <h2 class="mt-2 text-heading font-semibold tracking-[-0.015em] text-text">Work handed in for your review</h2>
    </header>
    <p class="text-small text-text-muted">
      A session wrote the wrapup and claimed this task as one piece. Accept it to file the task, or send it back for another pass.
    </p>
    <div class="flex gap-2">
      <AppButton variant="quiet" size="sm" data-testid="handed-in-send-back" @click="review.sendBack">Send back</AppButton>
      <AppButton variant="primary" size="sm" data-testid="handed-in-accept" @click="review.accept">Accept task</AppButton>
    </div>
  </section>
</template>
