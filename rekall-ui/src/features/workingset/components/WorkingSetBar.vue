<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { useRoute } from 'vue-router'
import WorkingSetProjectTabs from '@/features/workingset/components/WorkingSetProjectTabs.vue'
import WorkingSetTaskTabs from '@/features/workingset/components/WorkingSetTaskTabs.vue'
import { useWorkingSetKeys } from '@/features/workingset/composables/useWorkingSetKeys'
import { useWorkingSetNavigation } from '@/features/workingset/composables/useWorkingSetNavigation'
import { reviewCountsByTask } from '@/features/workingset/lib/review-counts'
import { useWorkingSetStore } from '@/features/workingset/store/workingSet.store'
import { useConsoleStore } from '@/stores/console.store'
import type { TaskId } from '@/model/branded'

/**
 * The two-level bar under the header: the projects with open tasks, then the open tasks of the
 * project in view. It shows on a task page and a project page while something is open, and the
 * project and task it marks follow the route, however the page was reached.
 */
const route = useRoute()
const workingSet = useWorkingSetStore()
const { reviewQueue } = storeToRefs(useConsoleStore())
const { taskInView, projectIdInView, placeToVisit, visit } = useWorkingSetNavigation()

const reviewCounts = computed(() => reviewCountsByTask(reviewQueue.value))
const groupInView = computed(() =>
  workingSet.groups.flatMap((company) => company.projects).find((group) => group.project.id === projectIdInView.value) ?? null
)
const shown = computed(() => (route.name === 'task' || route.name === 'project-detail') && workingSet.groups.length > 0)

useWorkingSetKeys()

async function close(taskId: TaskId): Promise<void> {
  const wasInView = taskInView.value?.id === taskId
  const landing = await workingSet.close(taskId)
  if (wasInView && landing) visit(landing)
}
</script>

<template>
  <div v-if="shown" class="shrink-0" data-testid="working-set-bar">
    <WorkingSetProjectTabs
      :groups="workingSet.groups"
      :current-project-id="projectIdInView"
      :review-counts="reviewCounts"
      @select="(projectId) => visit(placeToVisit(projectId))"
    />
    <WorkingSetTaskTabs
      v-if="groupInView"
      :project="groupInView.project"
      :tasks="groupInView.tasks"
      :current-task-id="taskInView?.id ?? null"
      :review-counts="reviewCounts"
      @select="visit"
      @close="close"
    />
  </div>
</template>
