<script setup lang="ts">
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import type { Project, Task } from '@/features/catalog/model/catalog'
import { TASK_STATUS_COLOR, TASK_STATUS_LABEL } from '@/features/catalog/model/task-status'
import TaskActivityMark from '@/features/workingset/components/TaskActivityMark.vue'
import { TASK_ACTIVITY_LABEL, type StepTally, type TaskActivity } from '@/features/workingset/model/task-activity'
import type { OpenPlace } from '@/features/workingset/model/working-set'
import type { TaskId } from '@/model/branded'

/**
 * Level 2 of the working-set bar: the project's task list, then a tab per open task. The dot is
 * the task's status, the bars its activity, the count its executed steps, amber while claims wait.
 * A live terminal adds a key that opens it, lit while on screen.
 */
const props = defineProps<{
  project: Project
  tasks: readonly Task[]
  currentTaskId: TaskId | null
  activities: ReadonlyMap<TaskId, TaskActivity>
  tallies: ReadonlyMap<TaskId, StepTally>
  reviewCounts: ReadonlyMap<TaskId, number>
  terminalTaskIds: ReadonlySet<TaskId>
  terminalShown: boolean
}>()

const emit = defineEmits<{ select: [place: OpenPlace]; close: [taskId: TaskId]; openTerminal: [taskId: TaskId] }>()

function selectTask(taskId: TaskId): void {
  emit('select', { projectId: props.project.id, taskId })
}

function selectList(): void {
  emit('select', { projectId: props.project.id, taskId: null })
}

function activityOf(task: Task): TaskActivity {
  return props.activities.get(task.id) ?? 'INACTIVE'
}

function tallyText(task: Task): string | null {
  const tally = props.tallies.get(task.id)
  return tally && tally.total > 0 ? `${tally.executed}/${tally.total}` : null
}

function waitingCount(task: Task): number {
  return props.reviewCounts.get(task.id) ?? 0
}

function isOnScreen(task: Task): boolean {
  return props.terminalShown && props.currentTaskId === task.id
}

function tallyTitle(task: Task): string {
  const tally = props.tallies.get(task.id)
  const executed = `${tally?.executed ?? 0} of ${tally?.total ?? 0} steps executed`
  return waitingCount(task) > 0 ? `${executed}, ${waitingCount(task)} waiting for your review` : executed
}
</script>

<template>
  <div
    class="flex h-[38px] shrink-0 items-stretch gap-0.5 overflow-x-auto overflow-y-hidden border-b border-border bg-canvas px-3"
    role="tablist"
    :aria-label="`${project.title} tasks`"
    data-testid="working-set-tasks"
  >
    <button
      type="button"
      role="tab"
      class="focus-ring relative flex shrink-0 cursor-pointer items-center gap-2 whitespace-nowrap pl-2.5 pr-3.5 font-medium transition-colors duration-(--motion-quick) hover:bg-surface hover:text-text"
      :class="currentTaskId === null ? 'text-text after:absolute after:inset-x-0 after:-bottom-px after:h-0.5 after:rounded-full after:bg-accent' : 'text-text-muted'"
      :aria-selected="currentTaskId === null"
      data-testid="working-set-all-tasks"
      @click="selectList"
    >
      All tasks
      <span class="text-caption tabular-nums text-text-subtle">{{ project.taskCount }}</span>
    </button>
    <span class="my-2.5 mx-1.5 w-px shrink-0 bg-border" aria-hidden="true"></span>
    <div
      v-for="task in tasks"
      :key="task.id"
      class="group/tab relative flex shrink-0 transition-colors duration-(--motion-quick) hover:bg-surface hover:text-text"
      :class="[
        terminalTaskIds.has(task.id) ? 'max-w-[278px]' : 'max-w-[250px]',
        currentTaskId === task.id ? 'text-text after:absolute after:inset-x-0 after:-bottom-px after:h-0.5 after:rounded-full after:bg-accent' : 'text-text-muted'
      ]"
      data-testid="working-set-task-tab"
    >
      <button
        v-if="terminalTaskIds.has(task.id)"
        type="button"
        class="focus-ring ml-1.5 grid size-[22px] shrink-0 place-items-center self-center rounded-key"
        :class="isOnScreen(task) ? 'key-lit' : 'key-quiet cursor-pointer'"
        :aria-pressed="isOnScreen(task)"
        :aria-label="`Open the terminal of ${task.code}`"
        :title="`Open the terminal of ${task.code}`"
        data-testid="working-set-task-terminal"
        @click="emit('openTerminal', task.id)"
      >
        <svg class="size-3.5" viewBox="0 0 14 14" fill="none" aria-hidden="true">
          <path d="M3 4l3 3-3 3M7.5 10.5H11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </button>
      <button
        type="button"
        role="tab"
        class="focus-ring flex min-w-0 cursor-pointer items-center gap-2 whitespace-nowrap pr-7"
        :class="terminalTaskIds.has(task.id) ? 'pl-2' : 'pl-3'"
        :aria-selected="currentTaskId === task.id"
        :title="`${task.code} ${task.title}, ${TASK_ACTIVITY_LABEL[activityOf(task)]}`"
        data-testid="working-set-task"
        @click="selectTask(task.id)"
      >
        <span class="flex shrink-0 items-center gap-1.5">
          <span
            class="size-2 shrink-0 rounded-full"
            :class="TASK_STATUS_COLOR[task.status]"
            role="img"
            :aria-label="TASK_STATUS_LABEL[task.status]"
          ></span>
          <TaskActivityMark :activity="activityOf(task)" />
        </span>
        <span class="shrink-0 font-mono text-caption text-text-subtle">{{ task.code }}</span>
        <span class="truncate">{{ task.title }}</span>
        <span
          v-if="tallyText(task)"
          class="shrink-0 font-mono text-caption tabular-nums"
          :class="waitingCount(task) > 0 ? 'rounded-full bg-accent px-1.5 font-medium text-accent-ink' : 'text-text-subtle'"
          :title="tallyTitle(task)"
          :aria-label="tallyTitle(task)"
          :data-waiting="waitingCount(task) > 0 ? 'true' : undefined"
          data-testid="working-set-task-tally"
        >
          {{ tallyText(task) }}
        </span>
      </button>
      <button
        type="button"
        class="focus-ring absolute right-1.5 top-1/2 grid size-[18px] -translate-y-1/2 cursor-pointer place-items-center rounded-key text-text-subtle opacity-0 transition-opacity duration-(--motion-quick) hover:bg-surface-hover hover:text-text focus-visible:opacity-100 group-hover/tab:opacity-100"
        :class="currentTaskId === task.id && 'opacity-100'"
        :aria-label="`Close ${task.code}`"
        :title="`Close ${task.code}`"
        data-testid="working-set-task-close"
        @click="emit('close', task.id)"
      >
        <CloseGlyph small />
      </button>
    </div>
    <button
      type="button"
      class="focus-ring ml-1 grid size-[26px] shrink-0 cursor-pointer place-items-center self-center rounded-key text-text-subtle transition-colors duration-(--motion-quick) hover:bg-surface-raised hover:text-text"
      aria-label="Open another task from the task list"
      title="Open another task from the task list"
      data-testid="working-set-add"
      @click="selectList"
    >
      <svg class="size-3.5" viewBox="0 0 14 14" fill="none" aria-hidden="true">
        <path d="M7 2.5v9M2.5 7h9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
      </svg>
    </button>
  </div>
</template>
