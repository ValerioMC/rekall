<script setup lang="ts">
import AppButton from '@/components/ui/AppButton.vue'
import { usePlanTask } from '@/features/claude/composables/usePlanTask'
import type { TaskId } from '@/model/branded'

/** Has a session go through the task with the person: types `/rk <anchor> refine` into the live session, or opens one on it. */
const props = defineProps<{ taskId: TaskId; anchor: string; folder: string | null }>()

const refining = usePlanTask(
  () => props.taskId,
  () => props.anchor,
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
    @click="refining.start('REFINE')"
  >
    Refine with Claude
  </AppButton>
</template>
