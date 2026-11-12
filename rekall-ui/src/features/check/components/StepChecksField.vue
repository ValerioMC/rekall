<script setup lang="ts">
import { computed, watch } from 'vue'
import CheckList from '@/features/check/components/CheckList.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useChecksStore } from '@/features/check/store/checks.store'
import { stepIsComplete, type TaskStep } from '@/features/steps/model/task-step'
import { type Task } from '@/features/catalog/model/catalog'

/**
 * The checks only this step has to pass, on top of the project's. Hidden on a finished step that
 * has none, since there is nothing left to add them to.
 */
const props = defineProps<{ step: TaskStep; task: Task }>()

const store = useChecksStore()
const { run } = useAsyncAction()

const checks = computed(() => store.stepChecksOf(props.task.projectId, props.step.id))
const shown = computed(() => checks.value.length > 0 || !stepIsComplete(props.step.state))

watch(
  () => props.task.projectId,
  (projectId) => void run(() => store.load(projectId)),
  { immediate: true }
)
</script>

<template>
  <div v-if="shown" class="mb-3" data-testid="step-checks">
    <span class="eyebrow">Done when</span>
    <CheckList
      :project-id="task.projectId"
      :step-id="step.id"
      :checks="checks"
      empty-text="Only the project's checks apply to this step."
    />
  </div>
</template>
