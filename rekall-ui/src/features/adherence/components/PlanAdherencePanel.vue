<script setup lang="ts">
import { toRef } from 'vue'
import PlanAdherenceFigures from '@/features/adherence/components/PlanAdherenceFigures.vue'
import { useTaskAdherence } from '@/features/adherence/composables/useTaskAdherence'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { TaskId } from '@/model/branded'

/** A task's adherence figures on a card of their own, read as the plan's steps change. */
const props = defineProps<{ taskId: TaskId; steps: readonly TaskStep[] }>()

const adherence = useTaskAdherence(toRef(props, 'taskId'), toRef(props, 'steps'))
</script>

<template>
  <div class="rounded-card border border-border bg-surface px-5 py-4">
    <PlanAdherenceFigures v-if="adherence" :adherence="adherence" />
  </div>
</template>
