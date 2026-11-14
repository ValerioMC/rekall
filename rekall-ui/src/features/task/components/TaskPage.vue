<script setup lang="ts">
import { computed, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useRoute, useRouter } from 'vue-router'
import AppButton from '@/components/ui/AppButton.vue'
import SubNav from '@/components/ui/SubNav.vue'
import TaskChangesTab from '@/features/task/components/TaskChangesTab.vue'
import TaskPageHeader from '@/features/task/components/TaskPageHeader.vue'
import TaskDeviationsTab from '@/features/task/components/TaskDeviationsTab.vue'
import TaskDescriptionTab from '@/features/task/components/TaskDescriptionTab.vue'
import TaskNotesTab from '@/features/task/components/TaskNotesTab.vue'
import TaskPlanTab from '@/features/task/components/TaskPlanTab.vue'
import TaskRail from '@/features/task/components/TaskRail.vue'
import TaskTimeTab from '@/features/task/components/TaskTimeTab.vue'
import TaskWidthMenu from '@/features/task/components/TaskWidthMenu.vue'
import WrapupPane from '@/features/wrapup/components/WrapupPane.vue'
import { taskTabItems } from '@/features/task/lib/task-tab-items'
import { tabOfPane, taskTabOf } from '@/features/task/model/task-tab'
import { isEpic } from '@/features/catalog/model/catalog'
import { workRows } from '@/features/work/lib/work-rows'
import { usePreferencesStore } from '@/features/settings/store/preferences.store'
import { useDeviationsStore } from '@/features/deviations/store/deviations.store'
import { useTokenUsageStore } from '@/features/tokens/store/tokenUsage.store'
import { useConsoleStore } from '@/stores/console.store'
import { asTaskId } from '@/model/branded'

/**
 * A task as a page at `/task/:id/:tab`. The route names the task: it is selected in the console
 * store, which the step, note and wrapup components read, and a task that does not exist says so.
 * The properties of the task sit in a bar on the left, and the tabs fill the rest.
 */
const store = useConsoleStore()
const route = useRoute()
const router = useRouter()
const deviations = useDeviationsStore()
const tokenUsage = useTokenUsageStore()
const preferences = usePreferencesStore()
const { isLoading, selectedTask, selectedTaskSteps, selectedTaskCommitReferences } = storeToRefs(store)

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
  return taskTabItems(
    task.id,
    {
      steps: selectedTaskSteps.value.filter((step) => step.state !== 'DRAFT').length,
      claimedSteps: selectedTaskSteps.value.filter((step) => step.state === 'CLAIMED').length,
      notes: task.documentCount,
      deviations: deviations.of(task.id).length,
      commits: selectedTaskCommitReferences.value.length
    },
    isEpic(task)
  )
})

watch(
  [taskId, isLoading],
  ([id, loading]) => {
    if (loading || !known.value || selectedTask.value?.id === id) return
    store.selectTask(id)
  },
  { immediate: true }
)

// An epic has no plan and changes no code itself, so its page opens on the description.
watch(
  [row, tab],
  ([shown, current]) => {
    if (shown && (current === 'plan' || current === 'changes') && isEpic(shown.task)) {
      void router.replace({ name: 'task', params: { id: shown.task.id, tab: 'description' } })
    }
  },
  { immediate: true }
)

// Search and the review queue still select a task and a pane in the store; the page follows to
// that task, on the tab that shows the pane.
watch(
  () => [store.selectedTaskId, store.paneFocus] as const,
  ([selected, pane]) => {
    if (selected === null) return
    const wanted = tabOfPane(pane)
    if (selected === taskId.value && (wanted === null || wanted === tab.value)) return
    void router.push({ name: 'task', params: { id: selected, tab: wanted ?? 'description' } })
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

    <div v-else-if="row" class="@container/task flex min-h-0 w-full min-w-0 flex-1 flex-col" data-testid="task-column">
      <TaskPageHeader :task="row.task" />
      <div class="flex min-h-0 flex-1 border-t border-border">
        <TaskRail :row="row" />
        <div class="flex min-h-0 min-w-0 flex-1 flex-col">
          <SubNav :model-value="tab" :items="tabs" label="Task" class="px-6">
            <template #trailing><TaskWidthMenu v-model="preferences.taskWidth" /></template>
          </SubNav>
          <TaskPlanTab v-if="tab === 'plan'" :task="row.task" />
          <TaskChangesTab v-else-if="tab === 'changes'" :task="row.task" />
          <WrapupPane v-else-if="tab === 'wrapup'" embedded />
          <TaskNotesTab v-else-if="tab === 'notes'" :task="row.task" />
          <TaskDescriptionTab v-else-if="tab === 'description'" :task="row.task" />
          <TaskTimeTab v-else-if="tab === 'time'" :task="row.task" />
          <TaskDeviationsTab v-else :task="row.task" />
        </div>
      </div>
    </div>
  </div>
</template>
