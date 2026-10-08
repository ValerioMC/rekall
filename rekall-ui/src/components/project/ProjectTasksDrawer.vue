<script setup lang="ts">
import { nextTick, onMounted, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import AppDialog from '@/components/ui/AppDialog.vue'
import { TASK_STATUS_COLOR, TASK_STATUS_LABEL } from '@/model/catalog'
import type { Task } from '@/model/catalog'
import type { TaskId } from '@/model/branded'

/** The project's tasks on demand: a panel from the right edge, closed by Escape or a click outside. */
defineProps<{ tasks: readonly Task[] }>()

const emit = defineEmits<{ close: []; open: [taskId: TaskId] }>()

const closeButton = ref<HTMLButtonElement | null>(null)

onMounted(async () => {
  await nextTick()
  closeButton.value?.focus()
})
</script>

<template>
  <AppDialog
    label="Tasks on this project"
    placement="right"
    panel-class="max-w-[420px]"
    data-testid="tasks-drawer"
    @dismiss="emit('close')"
  >
    <header class="flex items-center gap-3 border-b border-border px-5 py-4">
      <h2 class="flex-1 text-title font-semibold tracking-[-0.01em] text-text">
        Tasks <span class="ml-1 font-normal text-text-subtle">{{ tasks.length }}</span>
      </h2>
      <button
        ref="closeButton"
        type="button"
        class="key-quiet focus-ring grid size-8 place-items-center rounded-key"
        aria-label="Close tasks"
        data-testid="tasks-drawer-close"
        @click="emit('close')"
      >
        <CloseGlyph />
      </button>
    </header>
    <p v-if="!tasks.length" class="px-5 py-6 text-body text-text-subtle">
      No task on this project yet. Create one from the console.
    </p>
    <ul v-else class="min-h-0 flex-1 divide-y divide-border overflow-y-auto">
      <li v-for="task in tasks" :key="task.id" class="flex items-center gap-3 px-5 py-2.5">
        <span class="size-[7px] shrink-0 rounded-full" :class="TASK_STATUS_COLOR[task.status]" aria-hidden="true" />
        <span class="min-w-0 flex-1">
          <span class="block truncate text-body text-text">{{ task.title }}</span>
          <span class="block truncate font-mono text-micro text-anchor/80">{{ task.label }}</span>
        </span>
        <span class="shrink-0 text-caption text-text-subtle">{{ TASK_STATUS_LABEL[task.status] }}</span>
        <AppButton size="sm" variant="secondary" :data-testid="`open-task-${task.label}`" @click="emit('open', task.id)">
          Open
        </AppButton>
      </li>
    </ul>
  </AppDialog>
</template>
