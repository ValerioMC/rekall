<script setup lang="ts">
import LogCommitButton from '@/features/commits/components/LogCommitButton.vue'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskStep } from '@/features/steps/model/task-step'

/**
 * What an open step's state says above its detail, and the commit it lets a person log while a
 * session runs or after it claimed. The keys that act on the state sit in the panel header.
 */
defineProps<{ step: TaskStep; task: Task }>()

const BAR = 'mb-4 flex flex-wrap items-center gap-2 rounded-card border px-3 py-2'
</script>

<template>
  <div v-if="step.state === 'RUNNING'" :class="[BAR, 'border-accent/60 bg-accent-soft']" data-testid="step-running-bar">
    <span class="min-w-[min(100%,15rem)] flex-1 text-caption leading-snug text-text-muted">
      A session is on this step now. Log its latest commit here as it lands, without waiting for the
      step to be claimed.
    </span>
    <LogCommitButton variant="bar" :task-id="task.id" :step-id="step.id" :folder="task.projectRepoFolder" />
  </div>

  <div v-else-if="step.state === 'CLAIMED'" :class="[BAR, 'border-accent/25 bg-accent-soft/40']" data-testid="step-review-bar">
    <span class="min-w-[min(100%,15rem)] flex-1 text-caption leading-snug text-text-muted">
      A session claimed this. Accept it to tick the box, or send it back for another pass: what it
      worked from is sealed below, and the feedback goes underneath.
    </span>
    <LogCommitButton variant="bar" :task-id="task.id" :step-id="step.id" :folder="task.projectRepoFolder" />
  </div>

  <div v-else-if="step.state === 'DONE'" :class="[BAR, 'border-border']" data-testid="step-reopen-bar">
    <span class="min-w-[min(100%,15rem)] flex-1 text-caption leading-snug text-text-muted">
      Accepted. Reopening drops it back to open and clears the review it passed.
    </span>
  </div>
</template>
