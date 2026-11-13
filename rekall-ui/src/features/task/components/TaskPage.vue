<script setup lang="ts">
import { computed, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useRoute, useRouter } from 'vue-router'
import AppButton from '@/components/ui/AppButton.vue'
import SubNav from '@/components/ui/SubNav.vue'
import TaskPageHeader from '@/features/task/components/TaskPageHeader.vue'
import TaskPlanTab from '@/features/task/components/TaskPlanTab.vue'
import TaskTabInConsole from '@/features/task/components/TaskTabInConsole.vue'
import { taskTabItems } from '@/features/task/lib/task-tab-items'
import { taskTabOf } from '@/features/task/model/task-tab'
import { workRows } from '@/features/work/lib/work-rows'
import { useDeviationsStore } from '@/features/deviations/store/deviations.store'
import { useTokenUsageStore } from '@/features/tokens/store/tokenUsage.store'
import { useConsoleStore } from '@/stores/console.store'
import { asTaskId } from '@/model/branded'

/**
 * A task as a page at `/task/:id/:tab`. The route names the task: it is selected in the console
 * store, which the step, note and wrapup components read, and a task that does not exist says so.
 */
const store = useConsoleStore()
const route = useRoute()
const router = useRouter()
const deviations = useDeviationsStore()
const tokenUsage = useTokenUsageStore()
const { isLoading, selectedTask, selectedTaskSteps } = storeToRefs(store)

const taskId = computed(() => asTaskId(String(route.params.id)))
const tab = computed(() => taskTabOf(typeof route.params.tab === 'string' ? route.params.tab : undefined))
const known = computed(() => store.tasks.some((candidate) => candidate.id === taskId.value))
const row = computed(() => {
  const task = selectedTask.value
  if (!task || task.id !== taskId.value) return null
  return workRows([task], store.steps, (id) => tokenUsage.ofTask(id).generated)[0] ?? null
})
const tabs = computed(() => {
  const task = selectedTask.value
  if (!task) return []
  return taskTabItems(task.id, {
    steps: selectedTaskSteps.value.filter((step) => step.state !== 'DRAFT').length,
    claimedSteps: selectedTaskSteps.value.filter((step) => step.state === 'CLAIMED').length,
    notes: task.documentCount,
    deviations: deviations.of(task.id).length
  })
})

watch(
  [taskId, isLoading],
  ([id, loading]) => {
    if (loading || !known.value || selectedTask.value?.id === id) return
    store.selectTask(id)
  },
  { immediate: true }
)

// Search and the review queue still select a task in the store; the page follows to that task.
watch(
  () => store.selectedTaskId,
  (selected) => {
    if (selected === null || selected === taskId.value) return
    void router.push({ name: 'task', params: { id: selected, tab: 'plan' } })
  }
)
</script>

<template>
  <div class="flex min-h-0 min-w-0 flex-1 flex-col" data-testid="task-page">
    <div v-if="isLoading" class="px-6 pt-6" aria-hidden="true">
      <div class="skeleton h-4 w-48 rounded-control" />
      <div class="skeleton mt-4 h-7 w-2/3 rounded-control" />
    </div>

    <div v-else-if="!known" class="grid flex-1 place-items-center px-6" data-testid="task-missing">
      <div class="flex flex-col items-center gap-3 text-center">
        <p class="text-body font-medium text-text">No such task.</p>
        <p class="text-small text-text-muted">It may have been deleted, or the link is wrong.</p>
        <AppButton size="sm" @click="$router.push({ name: 'work' })">Back to work</AppButton>
      </div>
    </div>

    <template v-else-if="row">
      <TaskPageHeader :row="row" />
      <SubNav :model-value="tab" :items="tabs" label="Task" class="px-6" />
      <TaskPlanTab v-if="tab === 'plan'" :task="row.task" />
      <TaskTabInConsole v-else :tab="tab" />
    </template>
  </div>
</template>
