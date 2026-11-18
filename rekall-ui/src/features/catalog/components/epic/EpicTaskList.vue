<script setup lang="ts">
import { computed, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import StatusPill, { type StatusPillTone } from '@/components/ui/StatusPill.vue'
import TagBadge from '@/components/ui/TagBadge.vue'
import TaskMark from '@/features/catalog/components/TaskMark.vue'
import EpicTaskPicker from '@/features/catalog/components/epic/EpicTaskPicker.vue'
import { TASK_STATUS_LABEL, type TaskStatus } from '@/features/catalog/model/task-status'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskStep } from '@/features/steps/model/task-step'
import { taskMark } from '@/features/catalog/model/task-mark'
import type { TaskId } from '@/model/branded'

/** The tasks of the epic, oldest first, each with its status and how far its steps are. A row opens its task; its end takes it out of the epic. */
const props = defineProps<{ epic: Task; children: readonly Task[]; steps: readonly TaskStep[]; runningTaskIds: ReadonlySet<TaskId> }>()

const emit = defineEmits<{ open: [id: TaskId]; attach: [id: TaskId]; create: []; remove: [task: Task] }>()

const STATUS_TONE: Readonly<Record<TaskStatus, StatusPillTone>> = {
  IN_PROGRESS: 'progress',
  TODO: 'neutral',
  BLOCKED: 'danger',
  BACKLOG: 'neutral',
  DONE: 'done'
}

const isAttaching = ref(false)

const rows = computed(() =>
  props.children.map((task) => {
    const mark = taskMark(task, props.steps, props.runningTaskIds.has(task.id))
    const waitsOnYou = mark.state === 'CLAIMED' || mark.state === 'LIVE_CLAIMED'
    return { task, mark, waitsOnYou }
  })
)

function attach(id: TaskId): void {
  isAttaching.value = false
  emit('attach', id)
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col" aria-label="Tasks of this epic" data-testid="epic-task-list">
    <header class="flex h-11 shrink-0 items-center gap-2 border-b border-border pl-5 pr-3">
      <h2 class="eyebrow text-caption">Tasks</h2>
      <span class="rounded-key bg-canvas px-1.5 py-px font-mono text-micro tabular-nums text-text-subtle">{{ children.length }}</span>
      <span class="text-micro text-text-subtle">oldest first</span>
      <span class="ml-auto flex items-center gap-1">
        <AppButton variant="quiet" size="sm" :pressed="isAttaching || undefined" :aria-expanded="isAttaching" data-testid="epic-attach-task" @click="isAttaching = !isAttaching">
          Add existing
        </AppButton>
        <AppButton size="sm" data-testid="epic-create-task" @click="emit('create')"><span class="text-accent" aria-hidden="true">+</span> New task</AppButton>
      </span>
    </header>

    <div v-if="isAttaching" class="shrink-0 border-b border-border px-5 py-3">
      <EpicTaskPicker :epic="epic" @pick="attach" @close="isAttaching = false" />
    </div>

    <p v-if="!children.length" class="px-5 py-6 text-small leading-relaxed text-text-subtle">
      No task belongs to this epic yet. Add one that exists, or create a new one here.
    </p>

    <ul v-else class="page-scroll min-h-0 flex-1 divide-y divide-border">
      <li v-for="{ task, mark, waitsOnYou } in rows" :key="task.id" class="group/row flex items-start" :class="task.status === 'DONE' && 'filed-row'">
        <button
          class="focus-ring grid min-w-0 flex-1 grid-cols-[14px_minmax(0,1fr)_auto] items-center gap-x-2.5 gap-y-1.5 py-2.5 pl-5 pr-2 text-left transition-colors hover:bg-surface-hover"
          data-testid="epic-task-row"
          @click="emit('open', task.id)"
        >
          <span class="relative grid size-3.5 place-items-center">
            <TaskMark :state="mark.state" :status="task.status" :accepted="mark.accepted" :selected="false" />
          </span>
          <span class="flex min-w-0 items-center gap-2">
            <span class="min-w-0 truncate text-body font-medium text-text">{{ task.title }}</span>
            <TagBadge
              v-if="task.tagId && task.tagName && task.tagIcon && task.tagColor"
              class="shrink-0"
              :name="task.tagName"
              :icon="task.tagIcon"
              :color="task.tagColor"
            />
          </span>
          <StatusPill :tone="STATUS_TONE[task.status]" data-testid="epic-task-status">{{ TASK_STATUS_LABEL[task.status] }}</StatusPill>

          <span class="col-start-2 col-end-4 flex min-w-0 items-center gap-2.5">
            <span class="anchor-chip shrink-0 px-1.5 py-px text-micro leading-[15px]">{{ task.code }}</span>
            <template v-if="task.stepCount > 0">
              <span class="trough flex h-[6px] w-20 shrink-0 p-[1px]" :title="`${task.stepsDone} of ${task.stepCount} steps done`">
                <span
                  class="tube-gold h-full rounded-full"
                  :class="task.stepsDone === task.stepCount && 'tube-safe'"
                  :style="{ width: `${(task.stepsDone / task.stepCount) * 100}%` }"
                />
              </span>
              <span class="shrink-0 font-mono text-micro tabular-nums text-text-subtle">{{ task.stepsDone }}/{{ task.stepCount }}</span>
            </template>
            <span v-else class="shrink-0 text-micro text-text-subtle">No checklist</span>
            <span v-if="waitsOnYou" class="shrink-0 text-micro text-session-strong" data-testid="epic-task-claimed">claimed</span>
          </span>
        </button>
        <button
          type="button"
          class="focus-ring mr-3 mt-2 grid size-7 shrink-0 place-items-center rounded-control text-text-subtle opacity-0 transition-[opacity,color,background-color] duration-(--motion-quick) hover:bg-danger-soft hover:text-danger focus-visible:opacity-100 group-hover/row:opacity-100 group-focus-within/row:opacity-100"
          :title="`Take ${task.title} out of the epic`"
          :aria-label="`Take ${task.title} out of the epic`"
          data-testid="epic-remove-task"
          @click="emit('remove', task)"
        >
          <svg class="size-3" viewBox="0 0 12 12" fill="none" aria-hidden="true">
            <path d="M3 3l6 6M9 3l-6 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
          </svg>
        </button>
      </li>
    </ul>
  </section>
</template>
