<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { useRoute } from 'vue-router'
import WorkingSetProjectTabs from '@/features/workingset/components/WorkingSetProjectTabs.vue'
import WorkingSetTaskTabs from '@/features/workingset/components/WorkingSetTaskTabs.vue'
import { useWorkingSetKeys } from '@/features/workingset/composables/useWorkingSetKeys'
import { useWorkingSetNavigation } from '@/features/workingset/composables/useWorkingSetNavigation'
import { reviewCountsByTask } from '@/features/workingset/lib/review-counts'
import { stepTally, taskActivity } from '@/features/workingset/lib/task-activity'
import { useWorkingSetStore } from '@/features/workingset/store/workingSet.store'
import { useChatStore } from '@/features/chat/store/chat.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import type { StepTally, TaskActivity } from '@/features/workingset/model/task-activity'
import { useConsoleStore } from '@/stores/console.store'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { TaskId } from '@/model/branded'

/**
 * The two-level bar under the header: the projects with open tasks, then the open tasks of the
 * project in view. It shows on a task page and a project page while something is open, and the
 * project and task it marks follow the route, however the page was reached.
 */
const route = useRoute()
const workingSet = useWorkingSetStore()
const { reviewQueue, steps } = storeToRefs(useConsoleStore())
const chat = useChatStore()
const terminals = useTerminalStore()
const { taskInView, projectIdInView, placeToVisit, visit } = useWorkingSetNavigation()

const reviewCounts = computed(() => reviewCountsByTask(reviewQueue.value))
const groupInView = computed(() =>
  workingSet.groups.flatMap((company) => company.projects).find((group) => group.project.id === projectIdInView.value) ?? null
)
const tasksInView = computed(() => groupInView.value?.tasks ?? [])
const activities = computed(() => new Map<TaskId, TaskActivity>(tasksInView.value.map((task) => [task.id, activityOf(task.id)])))
const tallies = computed(() => new Map<TaskId, StepTally>(tasksInView.value.map((task) => [task.id, stepTally(stepsOf(task.id))])))
const shown = computed(() => (route.name === 'task' || route.name === 'project-detail') && workingSet.groups.length > 0)

useWorkingSetKeys()

function stepsOf(taskId: TaskId): TaskStep[] {
  return steps.value.filter((step) => step.taskId === taskId)
}

function activityOf(taskId: TaskId): TaskActivity {
  const chatSession = chat.chatForTask(taskId)
  return taskActivity({
    steps: stepsOf(taskId),
    turn: chatSession ? chat.timelineOf(chatSession.id).turn : null,
    hasLiveSession: terminals.terminalForTask(taskId) !== null
  })
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
      :activities="activities"
      :tallies="tallies"
      @select="visit"
      @close="close"
    />
  </div>
</template>
