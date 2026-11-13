<script setup lang="ts">
import { computed } from 'vue'
import StepCheckResults from '@/features/check/components/StepCheckResults.vue'
import StepChecksField from '@/features/check/components/StepChecksField.vue'
import CommitReferenceList from '@/features/commits/components/CommitReferenceList.vue'
import StepDetailEditor from '@/features/steps/components/StepDetailEditor.vue'
import StepPassHistory from '@/features/steps/components/StepPassHistory.vue'
import StepSkillsField from '@/features/steps/components/StepSkillsField.vue'
import StepStateBar from '@/features/steps/components/StepStateBar.vue'
import { useConsoleStore } from '@/stores/console.store'
import { stepIsComplete, type TaskStep } from '@/features/steps/model/task-step'
import type { Task } from '@/features/catalog/model/catalog'

/**
 * What an open step holds: its state bar, check results, commits, sealed passes, skills, checks
 * and the detail being written. The checklist row opens it under the step; the task page's
 * decision panel shows it beside the list.
 */
const props = defineProps<{ step: TaskStep; task: Task; reopenArmed: boolean }>()

defineEmits<{ accept: []; 'send-back': []; reopen: [] }>()

const store = useConsoleStore()
const commits = computed(() => store.stepCommitReferences(props.step.id))
</script>

<template>
  <div data-testid="step-detail">
    <StepStateBar
      :step="step"
      :task="task"
      :reopen-armed="reopenArmed"
      @accept="$emit('accept')"
      @send-back="$emit('send-back')"
      @reopen="$emit('reopen')"
    />

    <StepCheckResults :step="step" />

    <div v-if="commits.length" class="mb-3" data-testid="step-commits">
      <span class="eyebrow">Commits <span class="font-mono">{{ commits.length }}</span></span>
      <CommitReferenceList :references="commits" dense />
    </div>

    <StepPassHistory v-if="step.passes.length" :passes="step.passes" class="mb-3" />

    <StepSkillsField v-if="!stepIsComplete(step.state)" :step="step" :task="task" />

    <StepChecksField :step="step" :task="task" />

    <StepDetailEditor
      :step-title="step.title"
      testid-prefix="step"
      :label="step.passes.length ? `Feedback for pass ${step.passes.length + 1}` : 'Detail'"
      editor-height="300px"
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
