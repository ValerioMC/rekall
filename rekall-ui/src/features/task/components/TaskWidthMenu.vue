<script setup lang="ts">
import { computed, ref } from 'vue'
import PopoverMenu, { type PopoverMenuEntry } from '@/components/ui/PopoverMenu.vue'
import { TASK_WIDTHS, type TaskWidth } from '@/common/config/ui-preferences'
import { TASK_WIDTH_LABEL } from '@/features/task/model/task-width'

/** The width of the task page's column, measured from its centre. One choice holds for every tab and every task. */
const width = defineModel<TaskWidth>({ required: true })

const open = ref(false)
const entries = computed<readonly PopoverMenuEntry[]>(() => [
  { kind: 'heading', label: 'Page width' },
  ...TASK_WIDTHS.map((candidate) => ({
    kind: 'item' as const,
    id: candidate,
    label: TASK_WIDTH_LABEL[candidate],
    current: candidate === width.value
  }))
])

function choose(id: string): void {
  width.value = id as TaskWidth
}
</script>

<template>
  <PopoverMenu v-model:open="open" :entries="entries" label="Page width" align="right" @pick="choose">
    <template #trigger="{ toggle }">
      <button
        type="button"
        class="focus-ring grid size-7 cursor-pointer place-items-center rounded-key text-text-subtle transition-colors duration-(--motion-quick) hover:bg-surface-raised hover:text-text"
        :title="`Page width: ${TASK_WIDTH_LABEL[width].toLowerCase()}`"
        aria-label="Page width"
        data-popover-trigger
        data-testid="task-width"
        @click="toggle"
      >
        <svg class="size-4" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <path d="M2 3.5v9M14 3.5v9" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
          <path d="M5 8h6M5 8l1.6-1.6M5 8l1.6 1.6M11 8 9.4 6.4M11 8 9.4 9.6" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </button>
    </template>
  </PopoverMenu>
</template>
