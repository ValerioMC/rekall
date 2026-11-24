<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { useRoute, useRouter } from 'vue-router'
import WorkingSetProjectTabs from '@/features/workingset/components/WorkingSetProjectTabs.vue'
import WorkingSetTaskTabs from '@/features/workingset/components/WorkingSetTaskTabs.vue'
import { reviewCountsByTask } from '@/features/workingset/lib/review-counts'
import type { OpenPlace } from '@/features/workingset/model/working-set'
import { useWorkingSetStore } from '@/features/workingset/store/workingSet.store'
import { useConsoleStore } from '@/stores/console.store'
import type { ProjectId, TaskId } from '@/model/branded'

/**
 * The two-level bar under the header: the projects with open tasks, then the open tasks of the
 * project in view. It shows on a task page and a project page while something is open, and the
 * project and task it marks follow the route, however the page was reached.
 */
const route = useRoute()
const router = useRouter()
const workingSet = useWorkingSetStore()
const catalog = useConsoleStore()
const { tasks, reviewQueue } = storeToRefs(catalog)

const reviewCounts = computed(() => reviewCountsByTask(reviewQueue.value))
const onTaskPage = computed(() => route.name === 'task')
const onProjectPage = computed(() => route.name === 'project-detail')
const taskInView = computed(() => (onTaskPage.value ? (tasks.value.find((task) => task.id === route.params.id) ?? null) : null))
const projectIdInView = computed<ProjectId | null>(() => {
  if (taskInView.value) return taskInView.value.projectId
  return onProjectPage.value ? (String(route.params.id) as ProjectId) : null
})
const groupInView = computed(() =>
  workingSet.groups.flatMap((company) => company.projects).find((group) => group.project.id === projectIdInView.value) ?? null
)
const shown = computed(() => (onTaskPage.value || onProjectPage.value) && workingSet.groups.length > 0)

function placeToVisit(projectId: ProjectId): OpenPlace {
  const remembered = workingSet.placeOf(projectId)
  return remembered.taskId !== null && workingSet.isOpen(remembered.taskId) ? remembered : { projectId, taskId: null }
}

function visit(place: OpenPlace): void {
  if (place.taskId === null) void router.push(`/projects/${place.projectId}`)
  else void router.push({ name: 'task', params: { id: place.taskId, tab: 'description' } })
}

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
