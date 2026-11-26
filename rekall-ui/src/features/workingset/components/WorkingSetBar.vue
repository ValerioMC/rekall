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
import { useChatPaneShown } from '@/features/chat/composables/useChatPaneShown'
import { useChatStore } from '@/features/chat/store/chat.store'
import { useOpenSessionInPane } from '@/features/claude/composables/useOpenSessionInPane'
import { useTerminalPaneShown } from '@/features/claude/composables/useTerminalPaneShown'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import type { StepTally, TaskActivity } from '@/features/workingset/model/task-activity'
import { useConsoleStore } from '@/stores/console.store'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { SessionKind, Terminal } from '@/features/claude/model/terminal'
import type { TaskId } from '@/model/branded'

/**
 * The two-level bar under the header: the projects with open tasks, then the open tasks of the
 * project in view. It shows on a task page, a project page and a full-size session while something
 * is open, and marks the project and task on screen, however they were reached.
 */
const route = useRoute()
const workingSet = useWorkingSetStore()
const { reviewQueue, steps } = storeToRefs(useConsoleStore())
const chat = useChatStore()
const terminals = useTerminalStore()
const { taskInView, projectIdInView, placeToVisit, visit } = useWorkingSetNavigation()
const terminalShown = useTerminalPaneShown()
const chatShown = useChatPaneShown()
const openInPane = useOpenSessionInPane()

const reviewCounts = computed(() => reviewCountsByTask(reviewQueue.value))
const groupInView = computed(() =>
  workingSet.groups.flatMap((company) => company.projects).find((group) => group.project.id === projectIdInView.value) ?? null
)
const tasksInView = computed(() => groupInView.value?.tasks ?? [])
const activities = computed(() => new Map<TaskId, TaskActivity>(tasksInView.value.map((task) => [task.id, activityOf(task.id)])))
const tallies = computed(() => new Map<TaskId, StepTally>(tasksInView.value.map((task) => [task.id, stepTally(stepsOf(task.id))])))
const sessionKinds = computed(
  () =>
    new Map<TaskId, SessionKind>(
      tasksInView.value.flatMap((task) => {
        const session = keyedSession(task.id)
        return session ? [[task.id, session.kind]] : []
      })
    )
)
const shownKind = computed<SessionKind | null>(() => {
  if (terminalShown.value) return 'TERMINAL'
  return chatShown.value ? 'CHAT' : null
})
const onBarPage = computed(
  () => route.name === 'task' || route.name === 'project-detail' || terminalShown.value || chatShown.value
)
const shown = computed(() => onBarPage.value && workingSet.groups.length > 0)

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

/** The session a task's key opens: its live one, or its ended chat while that is still on screen. */
function keyedSession(taskId: TaskId): Terminal | null {
  return terminals.terminalForTask(taskId) ?? chatOnScreen(taskId)
}

function chatOnScreen(taskId: TaskId): Terminal | null {
  if (!chatShown.value) return null
  return terminals.terminals.find((session) => session.id === chat.activeChatId && session.taskId === taskId) ?? null
}

function openSession(taskId: TaskId): void {
  const session = keyedSession(taskId)
  if (!session) return
  if (taskInView.value?.id === taskId && shownKind.value === session.kind) return
  openInPane(session)
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
      :list-shown="route.name === 'project-detail'"
      :activities="activities"
      :tallies="tallies"
      :review-counts="reviewCounts"
      :session-kinds="sessionKinds"
      :shown-kind="shownKind"
      @select="visit"
      @close="close"
      @open-session="openSession"
    />
  </div>
</template>
