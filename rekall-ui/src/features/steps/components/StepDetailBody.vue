<script setup lang="ts">
import StepDetailEditor from '@/features/steps/components/StepDetailEditor.vue'
import StepPassHistory from '@/features/steps/components/StepPassHistory.vue'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { Task } from '@/features/catalog/model/catalog'

/**
 * What a step says, in the order it is read: the passes it was sent back from, then the detail
 * being written. What was found on it and what it runs with sit beside it, in the evidence.
 */
defineProps<{ step: TaskStep; task: Task }>()
</script>

<template>
  <div class="flex flex-col gap-6" data-testid="step-detail">
    <StepPassHistory v-if="step.passes.length" :passes="step.passes" />

    <StepDetailEditor
      :step-title="step.title"
      testid-prefix="step"
      :label="step.passes.length ? `Feedback for pass ${step.passes.length + 1}` : 'Detail'"
      editor-height="24rem"
      :placeholder="
        step.passes.length
          ? 'What the next pass has to change. Claude reads it after the sealed passes above.'
          : 'What this step has to do. Markdown, and Claude gets it while the step is open.'
      "
      :empty-text="step.passes.length ? 'No feedback yet.' : 'Nothing written under this step yet.'"
      :write-prompt="step.passes.length ? 'Write what the next pass has to change' : 'Write the detail'"
      :path-root="task.projectRepoFolder"
    />
  </div>
</template>
