<script setup lang="ts">
import type { TaskStepLine, TaskStepLineTone } from '@/features/project/model/project-task-list'

defineProps<{ line: TaskStepLine }>()

const TEXT_TONE: Readonly<Record<TaskStepLineTone, string>> = {
  running: 'text-session-strong',
  claimed: 'text-text',
  next: 'text-text-muted',
  summary: 'text-text-subtle'
}

const LEAD_TONE: Readonly<Record<TaskStepLineTone, string>> = {
  running: '',
  claimed: 'text-accent-strong',
  next: '',
  summary: ''
}
</script>

<template>
  <span class="truncate text-small" :class="TEXT_TONE[line.tone]" :title="line.text" data-testid="task-row-step">
    <span v-if="line.lead" class="mr-2 font-medium" :class="LEAD_TONE[line.tone]">{{ line.lead }}</span>{{ line.text }}
  </span>
</template>
