<script setup lang="ts">
import { computed, ref } from 'vue'
import PopoverMenu, { type PopoverMenuEntry } from '@/components/ui/PopoverMenu.vue'
import StatusPill from '@/components/ui/StatusPill.vue'
import { TASK_STATUS_LABEL, TASK_STATUSES, type TaskStatus } from '@/features/catalog/model/task-status'
import { WORK_KIND_LABEL, WORK_KIND_TONE, type WorkRow } from '@/features/work/model/work-view'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'

/**
 * The pill that names a task's state and changes its status from a menu. The pill says what the
 * work is doing now; the menu marks the stored status. Presses stop here so a row stays closed.
 */
const props = defineProps<{ row: WorkRow }>()

const emit = defineEmits<{ changed: [] }>()

const store = useConsoleStore()
const toast = useToastStore()
const { run } = useAsyncAction()

const open = ref(false)
const task = computed(() => props.row.task)
const entries = computed<readonly PopoverMenuEntry[]>(() =>
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
  if (outcome === null) return
  emit('changed')
  toast.notify(`Status set to ${TASK_STATUS_LABEL[status].toLowerCase()}`)
}
</script>

<template>
  <PopoverMenu v-model:open="open" :entries="entries" label="Change the status" @pick="setStatus" @click.stop>
    <template #trigger="{ toggle }">
      <button
        type="button"
        class="group/status focus-ring cursor-pointer rounded-key"
        title="Change status"
        aria-haspopup="menu"
        :aria-expanded="open"
        data-popover-trigger
        data-testid="task-status"
        @click="toggle"
        @keydown.enter.stop
      >
        <StatusPill :tone="WORK_KIND_TONE[row.kind]" :live="row.running" class="transition-colors duration-(--motion-quick) group-hover/status:brightness-125">
          {{ WORK_KIND_LABEL[row.kind] }}
          <svg
            class="-mr-0.5 size-2.5 opacity-0 transition-[opacity,transform] duration-(--motion-quick) group-hover/status:opacity-100 group-focus-visible/status:opacity-100 group-hover/row:opacity-70"
            :class="open && 'rotate-180 opacity-100'"
            viewBox="0 0 10 10"
            fill="none"
            aria-hidden="true"
          >
            <path d="m2 3.75 3 3 3-3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </StatusPill>
      </button>
    </template>
  </PopoverMenu>
</template>
