<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import type { TaskStatus } from '@/model/catalog'
import type { TaskId } from '@/model/branded'

const emit = defineEmits<{ close: [] }>()

const store = useConsoleStore()
const { tasks, selectedDocument } = storeToRefs(store)
const { run } = useAsyncAction()

const STATUS_DOT: Record<TaskStatus, string> = {
  IN_PROGRESS: 'bg-accent',
  TODO: 'bg-text-subtle',
  BLOCKED: 'bg-danger',
  DONE: 'bg-safe'
}

const attached = computed(() => new Set(selectedDocument.value?.tasks.map((task) => task.id) ?? []))

async function toggle(taskId: TaskId): Promise<void> {
  const document = selectedDocument.value
  if (!document) return
  const current = document.tasks.map((task) => task.id)
  if (attached.value.has(taskId)) {
    if (current.length === 1) return
    await run(() => store.saveNote(document.id, { taskIds: current.filter((id) => id !== taskId) }))
  } else {
    await run(() => store.saveNote(document.id, { taskIds: [...current, taskId] }))
  }
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    event.stopPropagation()
    emit('close')
  }
}

onMounted(() => window.addEventListener('keydown', onKeydown, true))
onUnmounted(() => window.removeEventListener('keydown', onKeydown, true))
</script>

<template>
  <div
    class="fixed inset-0 z-(--z-modal) grid place-items-center bg-black/60 p-5 backdrop-blur-sm"
    @click.self="emit('close')"
  >
    <div
      class="w-full max-w-[440px] overflow-hidden rounded-[var(--radius-card)] border border-border-strong bg-surface shadow-lift"
      role="dialog"
      aria-modal="true"
      aria-label="Attach this note to tasks"
    >
      <div class="p-5">
        <h2 class="mb-2 text-[16px] font-semibold text-text">
          Where does {{ selectedDocument?.title }} belong?
        </h2>
        <p class="text-[13px] leading-relaxed text-text-muted">
          A note can sit on as many tasks as it is relevant to. Loading any of them brings it
          along.
        </p>

        <div class="mt-3.5 flex max-h-[260px] flex-col gap-0.5 overflow-y-auto">
          <button
            v-for="task in tasks"
            :key="task.id"
            data-testid="attach-row"
            class="focus-ring flex w-full items-center gap-2.5 rounded-[var(--radius-control)] border px-2.5 py-2 text-left transition-colors"
            :class="
              attached.has(task.id)
                ? 'border-accent bg-accent-soft text-text'
                : 'border-transparent text-text-muted hover:bg-surface-raised hover:text-text'
            "
            :aria-pressed="attached.has(task.id)"
            @click="toggle(task.id)"
          >
            <span class="size-[7px] shrink-0 rounded-full" :class="STATUS_DOT[task.status]" aria-hidden="true" />
            <span class="min-w-0 flex-1">
              <span class="block truncate text-[12.5px]">{{ task.title }}</span>
              <span class="block truncate font-mono text-[10.5px] text-anchor/80">
                {{ task.projectLabel }}/{{ task.label }}
              </span>
            </span>
            <span v-if="attached.has(task.id)" class="font-mono text-[12px] text-accent">&check;</span>
          </button>
        </div>
      </div>

      <div class="flex justify-end gap-2 border-t border-border bg-canvas px-5 py-3">
        <AppButton variant="primary" size="sm" @click="emit('close')">Done</AppButton>
      </div>
    </div>
  </div>
</template>
