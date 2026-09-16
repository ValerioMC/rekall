<script setup lang="ts">
import { computed, ref } from 'vue'
import { storeToRefs } from 'pinia'
import NoteQuickPicker from '@/components/console/NoteQuickPicker.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { TaskId } from '@/model/branded'

/**
 * The header chrome that opens `NoteQuickPicker` under itself, next to Log commit / Run here,
 * so notes can be put on the task or taken off it from the description or the checklist. It
 * carries how many notes are on the task, read from the store rather than passed in, so it stays
 * right as the picker flips them.
 */
const props = defineProps<{ taskId: TaskId }>()

const store = useConsoleStore()
const { documents } = storeToRefs(store)

const host = ref<HTMLElement | null>(null)
const pickerOpen = ref(false)

const count = computed(
  () => documents.value.filter((document) => document.tasks.some((ref) => ref.id === props.taskId)).length
)
</script>

<template>
  <div ref="host" class="relative inline-flex h-7 shrink-0" data-testid="notes-group">
    <button
      type="button"
      class="focus-ring relative inline-flex items-center gap-1.5 rounded-[var(--radius-control)] border px-2.5 text-xs font-medium transition-all duration-150 active:translate-y-px"
      :class="
        pickerOpen
          ? 'border-border-strong bg-surface-raised text-text'
          : 'border-border bg-transparent text-text-muted hover:border-border-strong hover:bg-surface-raised hover:text-text'
      "
      :title="count ? `${count} note${count === 1 ? '' : 's'} on this task. Add or remove one.` : 'Put a note on this task'"
      aria-haspopup="dialog"
      :aria-expanded="pickerOpen"
      data-testid="notes-open"
      @click="pickerOpen = !pickerOpen"
    >
      <svg class="size-3.5" viewBox="0 0 12 12" fill="none" aria-hidden="true">
        <path d="M1.6 3h8.8M1.6 6h8.8M1.6 9h5.4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      </svg>
      <span>Notes</span>
      <span
        v-if="count"
        class="rounded-full px-1.5 font-mono text-[10px] tabular-nums leading-[15px]"
        :class="pickerOpen ? 'bg-accent-soft text-accent' : 'bg-surface-raised text-text-subtle'"
        data-testid="notes-count"
      >
        {{ count }}
      </span>
    </button>

    <NoteQuickPicker
      v-if="pickerOpen && host"
      :task-id="taskId"
      :anchor="host"
      @close="pickerOpen = false"
    />
  </div>
</template>
