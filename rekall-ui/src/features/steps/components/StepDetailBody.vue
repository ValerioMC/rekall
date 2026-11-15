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
 * What an open step holds, in the order it is read: the state it is in, the passes it was sent
 * back from, the detail being written, then what was found (check results, commits) and the
 * skills and checks that set the step up, folded because they are not what a decision reads.
 */
const props = defineProps<{ step: TaskStep; task: Task }>()

const store = useConsoleStore()
const commits = computed(() => store.stepCommitReferences(props.step.id))
const setupOpen = computed(() => props.step.state === 'OPEN')
const finished = computed(() => stepIsComplete(props.step.state))
</script>

<template>
  <div data-testid="step-detail">
    <StepStateBar :step="step" :task="task" />

    <StepPassHistory v-if="step.passes.length" :passes="step.passes" class="mb-4" />

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

    <StepCheckResults :step="step" class="mt-6" />

    <div v-if="commits.length" class="mt-6" data-testid="step-commits">
      <span class="eyebrow">Commits <span class="font-mono">{{ commits.length }}</span></span>
      <CommitReferenceList :references="commits" dense />
    </div>

    <details class="step-setup mt-6 border-y border-border" :open="setupOpen" data-testid="step-setup">
      <summary class="focus-ring flex min-h-10 cursor-pointer list-none items-center gap-2 rounded-key text-small text-text-muted hover:text-text">
        <svg class="step-setup-chevron size-2.5 shrink-0 text-text-subtle" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <path d="M9 6l6 6-6 6" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        <span class="font-medium text-text">Skills and checks</span>
        <span class="text-text-subtle">what this step runs with, and what it has to pass</span>
      </summary>
      <div class="pb-1 pt-2">
        <StepSkillsField v-if="!finished" :step="step" :task="task" />
        <StepChecksField :step="step" :task="task" />
      </div>
    </details>
  </div>
</template>

<style scoped>
.step-setup > summary::-webkit-details-marker {
  display: none;
}

.step-setup[open] .step-setup-chevron {
  transform: rotate(90deg);
}
</style>
