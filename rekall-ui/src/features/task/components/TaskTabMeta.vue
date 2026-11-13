<script setup lang="ts">
import { computed, ref } from 'vue'
import CopyAnchorButton from '@/components/console/CopyAnchorButton.vue'
import PopoverMenu, { type PopoverMenuEntry } from '@/components/ui/PopoverMenu.vue'
import StatusPill from '@/components/ui/StatusPill.vue'
import TaskWidthMenu from '@/features/task/components/TaskWidthMenu.vue'
import { TASK_STATUS_LABEL, TASK_STATUSES, type TaskStatus } from '@/features/catalog/model/task-status'
import { WORK_KIND_LABEL, WORK_KIND_TONE, type WorkRow } from '@/features/work/model/work-view'
import { rkCommand } from '@/common/format/rk-command'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { usePreferencesStore } from '@/features/settings/store/preferences.store'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'

/**
 * The right end of the tab strip: the status pill that changes the status, the copyable `/rk`
 * anchor, and the page width. They describe the task and the page, not one tab.
 */
const props = defineProps<{ row: WorkRow }>()

const store = useConsoleStore()
const toast = useToastStore()
const preferences = usePreferencesStore()
const { run } = useAsyncAction()

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

async function setStatus(id: string): Promise<void> {
  const status = id as TaskStatus
  const outcome = await run(() => store.setTaskStatus(task.value.id, status))
  if (outcome !== null) toast.notify(`Status set to ${TASK_STATUS_LABEL[status].toLowerCase()}`)
}
</script>

<template>
  <PopoverMenu v-model:open="statusMenuOpen" :entries="statusEntries" label="Change the status" align="right" @pick="setStatus">
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

  <span class="inline-flex items-center gap-1 @max-[40rem]/task:hidden">
    <span class="anchor-chip px-1.5 py-px font-mono text-caption" data-testid="task-anchor">{{ rkCommand(task.anchor) }}</span>
    <CopyAnchorButton :anchor="task.anchor" />
  </span>

  <TaskWidthMenu v-model="preferences.taskWidth" />
</template>
