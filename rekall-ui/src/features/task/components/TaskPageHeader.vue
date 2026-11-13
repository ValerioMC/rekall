<script setup lang="ts">
import { computed, ref } from 'vue'
import { RouterLink } from 'vue-router'
import CopyAnchorButton from '@/components/console/CopyAnchorButton.vue'
import PopoverMenu, { type PopoverMenuEntry } from '@/components/ui/PopoverMenu.vue'
import StatusPill from '@/components/ui/StatusPill.vue'
import LaunchClaudeCodeButton from '@/features/claude/components/LaunchClaudeCodeButton.vue'
import OpenTerminalButton from '@/features/claude/components/OpenTerminalButton.vue'
import TaskTokenChip from '@/features/tokens/components/TaskTokenChip.vue'
import { trackedSeconds } from '@/features/task/lib/tracked-seconds'
import { TASK_STATUS_LABEL, TASK_STATUSES, type TaskStatus } from '@/features/catalog/model/task-status'
import { WORK_KIND_LABEL, WORK_KIND_TONE, type WorkRow } from '@/features/work/model/work-view'
import { formatDuration } from '@/common/format/duration'
import { rkCommand } from '@/common/format/rk-command'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useNow } from '@/composables/useNow'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'

/**
 * What the task page says before its tabs: where the task sits (company and project), its title,
 * the status pill that changes the status, the copyable `/rk` anchor, time and tokens, and the
 * keys that start or watch its session.
 */
const props = defineProps<{ row: WorkRow }>()

const MINUTE_MS = 60_000

const store = useConsoleStore()
const toast = useToastStore()
const { run } = useAsyncAction()
const now = useNow(MINUTE_MS)

const statusMenuOpen = ref(false)
const task = computed(() => props.row.task)
const statusEntries = computed<readonly PopoverMenuEntry[]>(() =>
  TASK_STATUSES.map((status) => ({
    kind: 'item',
    id: status,
    label: TASK_STATUS_LABEL[status],
    current: status === task.value.status
  }))
)
const tracked = computed(() => trackedSeconds(store.selectedTaskEntries, now.value))

async function setStatus(id: string): Promise<void> {
  const status = id as TaskStatus
  const outcome = await run(() => store.setTaskStatus(task.value.id, status))
  if (outcome !== null) toast.notify(`Status set to ${TASK_STATUS_LABEL[status].toLowerCase()}`)
}
</script>

<template>
  <header class="px-6 pt-4" data-testid="task-header">
    <nav class="flex items-center gap-2 text-small text-text-muted" aria-label="Where this task is">
      <RouterLink :to="{ name: 'work' }" class="focus-ring rounded-key hover:text-text" data-testid="crumb-work">Work</RouterLink>
      <span class="text-text-subtle" aria-hidden="true">/</span>
      <RouterLink
        :to="{ name: 'project-detail', params: { id: task.projectId } }"
        class="focus-ring rounded-key hover:text-text"
        data-testid="crumb-project"
      >
        {{ task.companyName }} · {{ task.projectTitle }}
      </RouterLink>
    </nav>

    <h1 class="mt-2 text-heading font-semibold tracking-[-0.015em] text-text" data-testid="task-title">{{ task.title }}</h1>

    <div class="mt-2.5 flex flex-wrap items-center gap-x-3 gap-y-2 text-small text-text-muted">
      <PopoverMenu v-model:open="statusMenuOpen" :entries="statusEntries" label="Change the status" @pick="setStatus">
        <template #trigger="{ toggle }">
          <button
            type="button"
            class="focus-ring cursor-pointer rounded-full"
            title="Change status"
            data-popover-trigger
            data-testid="task-status"
            @click="toggle"
          >
            <StatusPill :tone="WORK_KIND_TONE[row.kind]" :live="row.running">{{ WORK_KIND_LABEL[row.kind] }}</StatusPill>
          </button>
        </template>
      </PopoverMenu>

      <span class="inline-flex items-center gap-1">
        <span class="anchor-chip px-1.5 py-px font-mono text-caption" data-testid="task-anchor">{{ rkCommand(task.anchor) }}</span>
        <CopyAnchorButton :anchor="task.anchor" />
      </span>

      <span class="flex-1" />

      <span v-if="tracked > 0" data-testid="task-tracked">
        <b class="font-semibold text-text">{{ formatDuration(tracked) }}</b> tracked
      </span>
      <TaskTokenChip :task="task" />
      <LaunchClaudeCodeButton
        :task-id="task.id"
        :anchors="task.anchor"
        :folder="task.projectRepoFolder"
        missing-hint="Set this project's folder on its page to open a session from it"
      />
      <OpenTerminalButton v-if="task.status !== 'DONE'" :task-id="task.id" :anchor="task.anchor" :folder="task.projectRepoFolder" />
    </div>
  </header>
</template>
