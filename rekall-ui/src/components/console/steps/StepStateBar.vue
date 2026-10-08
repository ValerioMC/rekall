<script setup lang="ts">
import AppButton from '@/components/ui/AppButton.vue'
import LogCommitButton from '@/components/console/LogCommitButton.vue'
import type { Task, TaskStep } from '@/model/catalog'

/**
 * What an open step's state lets a person do, above its detail: log a commit while a session
 * runs, accept or send back a claim, reopen an accepted step. An open step has no bar.
 */
defineProps<{
  step: TaskStep
  task: Task
  /** A reopen asks twice: the first click arms it. */
  reopenArmed: boolean
}>()

defineEmits<{ accept: []; 'send-back': []; reopen: [] }>()

const BAR = 'mb-3 flex flex-wrap items-center gap-2 rounded-card border px-3 py-2'
</script>

<template>
  <div v-if="step.state === 'RUNNING'" :class="[BAR, 'border-accent/60 bg-accent-soft']" data-testid="step-running-bar">
    <span class="min-w-0 flex-1 text-caption leading-snug text-text-muted">
      A session is on this step now. Log its latest commit here as it lands, without waiting for the
      step to be claimed.
    </span>
    <LogCommitButton variant="bar" :task-id="task.id" :step-id="step.id" :folder="task.projectRepoFolder" />
  </div>

  <div v-else-if="step.state === 'CLAIMED'" :class="[BAR, 'border-accent/25 bg-accent-soft/40']" data-testid="step-review-bar">
    <span class="min-w-0 flex-1 text-caption leading-snug text-text-muted">
      A session claimed this. Accept it to tick the box, or send it back for another pass: what it
      worked from is sealed above, and the feedback goes underneath.
    </span>
    <LogCommitButton variant="bar" :task-id="task.id" :step-id="step.id" :folder="task.projectRepoFolder" />
    <AppButton
      variant="primary"
      size="sm"
      data-testid="step-accept"
      @click="$emit('accept')"
    >
      Accept
    </AppButton>
    <AppButton
      variant="quiet"
      size="sm"
      data-testid="step-send-back"
      @click="$emit('send-back')"
    >
      Send back
    </AppButton>
  </div>

  <div v-else-if="step.state === 'DONE'" :class="[BAR, 'border-border']" data-testid="step-reopen-bar">
    <span class="min-w-0 flex-1 text-caption leading-snug text-text-muted">
      Accepted. Reopening drops it back to open and clears the review it passed.
    </span>
    <button
      class="focus-ring h-7 shrink-0 rounded-key px-3 text-caption font-medium"
      :class="reopenArmed ? 'key-danger' : 'key-quiet'"
      data-testid="step-reopen"
      @click="$emit('reopen')"
    >
      {{ reopenArmed ? 'Confirm reopen' : 'Reopen' }}
    </button>
  </div>
</template>
