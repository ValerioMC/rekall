<script setup lang="ts">
import AppButton from '@/components/ui/AppButton.vue'
import { usePlanTask } from '@/features/claude/composables/usePlanTask'
import type { TaskId } from '@/model/branded'

const props = defineProps<{ taskId: TaskId; folder: string | null }>()

const refining = usePlanTask(
  () => props.taskId,
  () => props.folder,
  "Set this project's folder on its page to refine from it"
)
</script>

<template>
  <AppButton
    variant="primary"
    size="sm"
    title="Have a session go through this task with you and record each decision here"
    data-testid="decision-refine"
    :aria-disabled="refining.isRunning.value"
    :aria-busy="refining.isRunning.value"
    @click="refining.start('REFINE')"
  >
    Refine
  </AppButton>
</template>
