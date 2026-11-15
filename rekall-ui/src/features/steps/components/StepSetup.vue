<script setup lang="ts">
import { computed } from 'vue'
import StepChecksField from '@/features/check/components/StepChecksField.vue'
import StepSkillsField from '@/features/steps/components/StepSkillsField.vue'
import { useChecksStore } from '@/features/check/store/checks.store'
import { stepIsComplete, type TaskStep } from '@/features/steps/model/task-step'
import type { Task } from '@/features/catalog/model/catalog'

/**
 * What the step runs with and what it has to pass, side by side. A finished step has nothing
 * left to set up, so it shows only checks it already carries and nothing at all when it has none.
 */
const props = defineProps<{ step: TaskStep; task: Task; stacked?: boolean }>()

const checks = useChecksStore()
const finished = computed(() => stepIsComplete(props.step.state))
const shown = computed(() => !finished.value || checks.stepChecksOf(props.task.projectId, props.step.id).length > 0)
</script>

<template>
  <section
    v-if="shown"
    class="grid gap-x-10 gap-y-4"
    :class="stacked ? 'grid-cols-1' : 'grid-cols-[repeat(auto-fit,minmax(20rem,1fr))] border-t border-border pt-5'"
    aria-label="Setup"
    data-testid="step-setup"
  >
    <StepSkillsField v-if="!finished" :step="step" :task="task" />
    <StepChecksField :step="step" :task="task" />
  </section>
</template>
