<script setup lang="ts">
import LaunchClaudeCodeButton from '@/features/claude/components/LaunchClaudeCodeButton.vue'
import OpenTerminalButton from '@/features/claude/components/OpenTerminalButton.vue'
import LogCommitButton from '@/features/commits/components/LogCommitButton.vue'
import type { Task } from '@/features/catalog/model/catalog'

/**
 * What a person does to a task from its page, in the order the work goes: log a commit, open the
 * project in their own terminal, and the gold key that runs the task in the console. Each is
 * offered once, here; the tabs below do not repeat them.
 */
defineProps<{ task: Task }>()
</script>

<template>
  <div class="flex shrink-0 items-center gap-1.5" data-testid="task-actions">
    <LogCommitButton :task-id="task.id" :folder="task.projectRepoFolder" />
    <LaunchClaudeCodeButton
      :task-id="task.id"
      :anchors="task.anchor"
      :folder="task.projectRepoFolder"
      missing-hint="Set this project's folder on its page to open a session from it"
    />
    <OpenTerminalButton v-if="task.status !== 'DONE'" :task-id="task.id" :anchor="task.anchor" :folder="task.projectRepoFolder" />
  </div>
</template>
