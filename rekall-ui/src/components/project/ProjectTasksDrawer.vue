<script setup lang="ts">
import { nextTick, onMounted, onUnmounted, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import { trapTabKey } from '@/common/a11y/focus-trap'
import { useModalGate } from '@/composables/useModalGate'
import { TASK_STATUS_COLOR, TASK_STATUS_LABEL } from '@/model/catalog'
import type { Task } from '@/model/catalog'
import type { TaskId } from '@/model/branded'

/** The project's tasks on demand: a panel from the right edge, closed by Escape or a click outside. */
defineProps<{ tasks: readonly Task[] }>()

const emit = defineEmits<{ close: []; open: [taskId: TaskId] }>()

const { open: openModal, close: closeModal } = useModalGate()
const panel = ref<HTMLElement | null>(null)
const closeButton = ref<HTMLButtonElement | null>(null)

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    event.stopPropagation()
    emit('close')
    return
  }
  if (panel.value) trapTabKey(panel.value, event)
}

onMounted(async () => {
  openModal()
  window.addEventListener('keydown', onKeydown, true)
  await nextTick()
  closeButton.value?.focus()
})

onUnmounted(() => {
  closeModal()
  window.removeEventListener('keydown', onKeydown, true)
})
</script>

<template>
  <div class="fixed inset-0 z-(--z-modal) bg-black/50 backdrop-blur-[2px]" data-testid="tasks-drawer" @click.self="emit('close')">
    <aside
      ref="panel"
      class="slide-over-panel absolute inset-y-0 right-0 flex w-full max-w-[420px] flex-col border-l border-border-strong bg-surface shadow-lift"
      role="dialog"
      aria-modal="true"
      aria-label="Tasks on this project"
    >
      <header class="flex items-center gap-3 border-b border-border px-5 py-4">
        <h2 class="flex-1 text-[15px] font-semibold tracking-[-0.01em] text-text">
          Tasks <span class="ml-1 font-normal text-text-subtle">{{ tasks.length }}</span>
        </h2>
        <button
          ref="closeButton"
          type="button"
          class="key-quiet focus-ring grid size-8 place-items-center rounded-[3px]"
          aria-label="Close tasks"
          data-testid="tasks-drawer-close"
          @click="emit('close')"
        >
          <CloseGlyph />
        </button>
      </header>
      <p v-if="!tasks.length" class="px-5 py-6 text-[13px] text-text-subtle">
        No task on this project yet. Create one from the console.
      </p>
      <ul v-else class="min-h-0 flex-1 divide-y divide-border overflow-y-auto">
        <li v-for="task in tasks" :key="task.id" class="flex items-center gap-3 px-5 py-2.5">
          <span class="size-[7px] shrink-0 rounded-full" :class="TASK_STATUS_COLOR[task.status]" aria-hidden="true" />
          <span class="min-w-0 flex-1">
            <span class="block truncate text-[13px] text-text">{{ task.title }}</span>
            <span class="block truncate font-mono text-[10.5px] text-anchor/80">{{ task.label }}</span>
          </span>
          <span class="shrink-0 text-[11px] text-text-subtle">{{ TASK_STATUS_LABEL[task.status] }}</span>
          <AppButton size="sm" variant="secondary" :data-testid="`open-task-${task.label}`" @click="emit('open', task.id)">
            Open
          </AppButton>
        </li>
      </ul>
    </aside>
  </div>
</template>
