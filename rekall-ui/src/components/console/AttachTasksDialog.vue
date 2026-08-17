<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { trapTabKey } from '@/common/a11y/focus-trap'
import type { TaskStatus } from '@/model/catalog'
import type { TaskId } from '@/model/branded'

const emit = defineEmits<{ close: [] }>()

const store = useConsoleStore()
const { tasks, selectedDocument } = storeToRefs(store)
const { run } = useAsyncAction()
const panel = ref<HTMLElement | null>(null)

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
    return
  }
  if (panel.value) trapTabKey(panel.value, event)
}

onMounted(async () => {
  window.addEventListener('keydown', onKeydown, true)
  await nextTick()
  panel.value?.querySelector<HTMLElement>('[data-testid="attach-row"], button')?.focus()
})
onUnmounted(() => window.removeEventListener('keydown', onKeydown, true))
</script>

<template>
  <div
    class="fixed inset-0 z-(--z-modal) grid place-items-center bg-black/60 p-5 backdrop-blur-sm"
    @click.self="emit('close')"
  >
    <div
      ref="panel"
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
              <span class="mt-0.5 flex items-center gap-1 truncate">
                <span class="shrink-0 text-[10px] text-text-subtle">{{ task.projectLabel }}</span>
                <span class="anchor-chip shrink-0 truncate px-1.5 py-px text-[9.5px] leading-[15px]">
                  {{ task.label }}
                </span>
              </span>
            </span>
            <svg
              v-if="attached.has(task.id)"
              class="size-3.5 shrink-0 text-accent"
              viewBox="0 0 16 16"
              fill="none"
              aria-hidden="true"
            >
              <path
                d="M6.5 9.5a2.6 2.6 0 0 0 3.7.2l1.6-1.6a2.6 2.6 0 0 0-3.7-3.7L7 5.3"
                stroke="currentColor"
                stroke-width="1.4"
                stroke-linecap="round"
              />
              <path
                d="M9.5 6.5a2.6 2.6 0 0 0-3.7-.2l-1.6 1.6a2.6 2.6 0 0 0 3.7 3.7L9 10.7"
                stroke="currentColor"
                stroke-width="1.4"
                stroke-linecap="round"
              />
            </svg>
          </button>
        </div>
      </div>

      <div class="flex justify-end gap-2 border-t border-border bg-canvas px-5 py-3">
        <AppButton variant="primary" size="sm" @click="emit('close')">Done</AppButton>
      </div>
    </div>
  </div>
</template>
