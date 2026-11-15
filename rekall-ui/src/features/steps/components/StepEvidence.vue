<script setup lang="ts">
import { computed } from 'vue'
import StepCheckResults from '@/features/check/components/StepCheckResults.vue'
import StepChecksField from '@/features/check/components/StepChecksField.vue'
import CommitReferenceList from '@/features/commits/components/CommitReferenceList.vue'
import LogCommitButton from '@/features/commits/components/LogCommitButton.vue'
import StepSkillsField from '@/features/steps/components/StepSkillsField.vue'
import { useConsoleStore } from '@/stores/console.store'
import { stepIsComplete, type TaskStep } from '@/features/steps/model/task-step'
import type { Task } from '@/features/catalog/model/catalog'

/**
 * What was found on a step and what it runs with, beside what it says: the last run of each
 * check, the commits logged against it (one can be logged while a session runs or after it
 * claimed, without waiting for the claim), and the skills and checks that set it up.
 */
const props = defineProps<{ step: TaskStep; task: Task }>()

const store = useConsoleStore()
const commits = computed(() => store.stepCommitReferences(props.step.id))
const canLogCommit = computed(() => props.step.state === 'RUNNING' || props.step.state === 'CLAIMED')
const finished = computed(() => stepIsComplete(props.step.state))
</script>

<template>
  <StepCheckResults :step="step" />

  <section v-if="commits.length || canLogCommit" data-testid="step-commits">
    <span class="eyebrow">Commits <span v-if="commits.length" class="font-mono">{{ commits.length }}</span></span>
    <CommitReferenceList v-if="commits.length" :references="commits" dense />
    <LogCommitButton v-if="canLogCommit" variant="bar" class="mt-1.5" :task-id="task.id" :step-id="step.id" :folder="task.projectRepoFolder" />
  </section>

  <section data-testid="step-setup" aria-label="Setup">
    <StepSkillsField v-if="!finished" :step="step" :task="task" />
    <StepChecksField :step="step" :task="task" />
  </section>
</template>
