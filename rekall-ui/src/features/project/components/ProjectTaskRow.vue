<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import CopyAnchorButton from '@/components/console/CopyAnchorButton.vue'
import PlanTube from '@/components/ui/PlanTube.vue'
import TagBadge from '@/components/ui/TagBadge.vue'
import TaskMark from '@/features/catalog/components/TaskMark.vue'
import { formatDuration } from '@/common/format/duration'
import { formatTokens } from '@/common/format/token-count'
import { taskMark } from '@/features/catalog/model/task-mark'
import { isEpic } from '@/features/catalog/model/catalog'
import { ageLabel } from '@/features/task/lib/age-label'
import type { ListedTask } from '@/features/project/model/project-task-list'
import type { TaskColumnId } from '@/features/project/lib/task-columns'
import { useConsoleStore } from '@/stores/console.store'
import { useProjectTasksStore } from '@/features/project/store/projectTasks.store'
import TaskDeleteButton from '@/features/task/components/TaskDeleteButton.vue'
import TaskStatusMenu from '@/features/task/components/TaskStatusMenu.vue'
import ProjectTaskDelivery from '@/features/project/components/ProjectTaskDelivery.vue'
import ProjectTaskPeek from '@/features/project/components/ProjectTaskPeek.vue'
import ProjectTaskStepLine from '@/features/project/components/ProjectTaskStepLine.vue'
import ProjectTaskTitle from '@/features/project/components/ProjectTaskTitle.vue'

const props = defineProps<{
  row: ListedTask
  now: number
  /** Set on an epic's row: the tasks it gathers, and whether they are folded away. */
  epic?: { readonly members: readonly ListedTask[]; readonly folded: boolean }
  /** Set on a task listed under its epic. */
  nested?: boolean
  /** The epic of a task whose epic is not listed with it. */
  epicTitle?: string | null
}>()

const emit = defineEmits<{ toggleEpic: [] }>()

const projectTasks = useProjectTasksStore()
const { columns, density, previewsBeside, peekedTaskId, previewed } = storeToRefs(projectTasks)
const { steps } = storeToRefs(useConsoleStore())

const task = computed(() => props.row.task)
const shownColumns = computed(() => new Set<TaskColumnId>(columns.value.map((column) => column.id)))
const mark = computed(() => taskMark(task.value, steps.value, false))
const filed = computed(() => props.row.kind === 'filed')
const hasPlan = computed(() => task.value.stepCount > 0)
const reviewLabel = computed(() =>
  props.row.kind === 'review' && props.row.claimedSteps > 0 ? `${props.row.claimedSteps} to review` : undefined
)
const expanded = computed(() => !previewsBeside.value && peekedTaskId.value === task.value.id)
const selected = computed(() => expanded.value || (previewsBeside.value && previewed.value?.task.id === task.value.id))
const memberSummary = computed(() => {
  const members = props.epic?.members ?? []
  const done = members.filter((member) => member.kind === 'filed').length
  const noun = members.length === 1 ? 'task' : 'tasks'
  return done > 0 ? `${members.length} ${noun} · ${done} done` : `${members.length} ${noun}`
})

function activate(): void {
  if (previewsBeside.value) projectTasks.preview(task.value.id)
  else projectTasks.togglePeek(task.value.id)
}
</script>

<template>
  <div>
    <div
      class="ledger-columns group/row focus-ring relative cursor-pointer rounded-control border-b border-border/60 transition-colors duration-(--motion-quick)"
      :class="[
        density === 'compact' ? 'h-[34px]' : 'h-11',
        filed ? 'text-filed' : 'text-text-muted',
        epic && 'bg-epic-soft/40',
        selected ? 'bg-surface-raised text-text' : 'hover:bg-surface hover:text-text',
        expanded && 'rounded-b-none shadow-[inset_0_0_0_1px_var(--color-border-strong)]'
      ]"
      role="button"
      tabindex="0"
      :aria-expanded="previewsBeside ? undefined : expanded"
      :aria-current="previewsBeside && selected ? 'true' : undefined"
      data-testid="task-row"
      @click="activate"
      @keydown.enter.self="activate"
    >
      <button
        v-if="epic"
        type="button"
        class="focus-ring grid size-6 cursor-pointer place-items-center rounded-key text-epic hover:bg-surface-hover"
        :aria-expanded="!epic.folded"
        :aria-label="epic.folded ? `Show the ${memberSummary} of this epic` : 'Fold this epic'"
        data-testid="task-epic-toggle"
        @click.stop="emit('toggleEpic')"
        @keydown.enter.stop
      >
        <svg class="size-3 transition-transform duration-(--motion-move)" :class="!epic.folded && 'rotate-90'" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <path d="m4.5 2.5 3.5 3.5-3.5 3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </button>
      <button
        v-else
        type="button"
        class="focus-ring grid size-6 cursor-pointer place-items-center rounded-key text-text-subtle hover:bg-surface-hover hover:text-text"
        :aria-label="`Preview ${task.code}`"
        :aria-expanded="previewsBeside ? undefined : expanded"
        data-testid="task-row-preview"
        @click.stop="activate"
        @keydown.enter.stop
      >
        <svg class="size-3 transition-transform duration-(--motion-move)" :class="expanded && 'rotate-90'" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <path d="m4.5 2.5 3.5 3.5-3.5 3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </button>

      <span class="relative grid size-3.5 place-items-center">
        <span v-if="nested" class="absolute -top-[22px] bottom-1/2 left-1/2 w-px bg-epic/35" aria-hidden="true" />
        <TaskMark :state="mark.state" :status="task.status" :accepted="mark.accepted" />
      </span>

      <span class="truncate font-mono text-caption" :class="filed ? 'text-filed' : 'text-anchor'" data-testid="task-row-code">{{ task.code }}</span>

      <ProjectTaskTitle :task="task" :filed="filed" :is-epic-row="epic !== undefined" :nested="nested" :epic-title="epicTitle" />

      <ProjectTaskStepLine v-if="shownColumns.has('step')" :line="row.line" />

      <span v-if="shownColumns.has('tag')" class="flex min-w-0" data-testid="task-row-tag">
        <TagBadge v-if="task.tagName && task.tagIcon && task.tagColor" :name="task.tagName" :icon="task.tagIcon" :color="task.tagColor" />
      </span>

      <span>
        <TaskStatusMenu :row="row" :label="reviewLabel" @changed="projectTasks.linger(task.id)" />
      </span>

      <span class="flex min-w-0 items-center gap-2.5 font-mono text-caption" data-testid="task-row-plan">
        <template v-if="hasPlan">
          <PlanTube class="w-26" :done="task.stepsDone" :claimed="row.claimedSteps" :total="task.stepCount" :live="row.running" />
          {{ task.stepsDone }}/{{ task.stepCount }}
        </template>
        <span v-else-if="epic" class="truncate text-epic" data-testid="task-epic-summary">{{ memberSummary }}</span>
        <span v-else class="text-text-subtle">{{ isEpic(task) ? 'epic' : 'no steps' }}</span>
      </span>

      <ProjectTaskDelivery v-if="shownColumns.has('delivery')" :task="task" />

      <span v-if="shownColumns.has('notes')" class="text-right font-mono text-caption tabular-nums text-text-subtle" data-testid="task-row-notes">
        {{ task.documentCount || '' }}
      </span>
      <span v-if="shownColumns.has('tokens')" class="text-right font-mono text-caption tabular-nums text-text-subtle" data-testid="task-row-tokens">
        {{ row.tokens > 0 ? formatTokens(row.tokens) : '' }}
      </span>
      <span v-if="shownColumns.has('time')" class="whitespace-nowrap text-right font-mono text-caption tabular-nums text-text-subtle" data-testid="task-row-time">
        {{ row.trackedSeconds > 0 ? formatDuration(row.trackedSeconds) : '' }}
      </span>
      <span class="text-right font-mono text-caption tabular-nums text-text-subtle">{{ ageLabel(task.updatedAt, now) }}</span>

      <span
        class="flex justify-end opacity-0 transition-opacity duration-(--motion-quick) focus-within:opacity-100 group-hover/row:opacity-100 has-[[role=alertdialog]]:opacity-100"
        :class="selected && 'opacity-100'"
        data-testid="task-row-actions"
        @click.stop
        @keydown.stop
      >
        <CopyAnchorButton :anchor="task.anchor" />
        <TaskDeleteButton :task="task" :leaves-to-project="false" />
      </span>
    </div>
    <div
      v-if="expanded"
      class="mb-1.5 rounded-b-control bg-surface-raised px-3 pb-3.5 pl-[66px] pt-1.5 shadow-[inset_0_0_0_1px_var(--color-border-strong)]"
    >
      <ProjectTaskPeek :row="row" />
    </div>
  </div>
</template>
