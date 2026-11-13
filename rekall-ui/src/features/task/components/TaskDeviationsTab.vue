<script setup lang="ts">
import { computed, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useRouter } from 'vue-router'
import DeviationPane from '@/features/deviations/components/DeviationPane.vue'
import { useTaskDeviations } from '@/features/deviations/composables/useTaskDeviations'
import { useConsoleStore } from '@/stores/console.store'
import type { Task } from '@/features/catalog/model/catalog'

/** The Deviations tab: where the work left the plan, one step at a time. It opens on the first deviation. */
const props = defineProps<{ task: Task }>()

const store = useConsoleStore()
const router = useRouter()
const { selectedDeviationStep } = storeToRefs(store)
const { deviations } = useTaskDeviations(computed(() => props.task.id))

watch(
  deviations,
  (all) => {
    const first = all[0]
    if (first && !all.some((deviation) => deviation.stepNumber === selectedDeviationStep.value)) store.openDeviation(first.stepNumber)
  },
  { immediate: true }
)

function showStepOnPlan(): void {
  void router.push({ name: 'task', params: { id: props.task.id, tab: 'plan' } })
}
</script>

<template>
  <DeviationPane embedded @open-step="showStepOnPlan" />
</template>
