<script setup lang="ts">
import { computed } from 'vue'
import StepCheckResults from '@/features/check/components/StepCheckResults.vue'
import CommitReferenceList from '@/features/commits/components/CommitReferenceList.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { TaskStep } from '@/features/steps/model/task-step'

/** What was found on a step: the last run of each check and the commits logged against it. Each shows only when there is something. */
const props = defineProps<{ step: TaskStep }>()

const store = useConsoleStore()
const commits = computed(() => store.stepCommitReferences(props.step.id))
</script>

<template>
  <StepCheckResults :step="step" />

  <section v-if="commits.length" data-testid="step-commits">
    <span class="eyebrow">Commits <span class="font-mono">{{ commits.length }}</span></span>
    <CommitReferenceList :references="commits" dense />
  </section>
</template>
