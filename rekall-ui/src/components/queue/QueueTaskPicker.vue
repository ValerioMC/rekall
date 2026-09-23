<script setup lang="ts">
/**
 * Adds a task to the queue by typing: its title, its label or its anchor, the same three ways the
 * console finds a task anywhere else. Arrow keys move through the matches, Enter adds the one
 * lit, Escape clears. A task already waiting or running in the queue is not offered; one whose
 * work is all claimed still is, and says so, because the queue will only skip it.
 */
import { computed, ref, useId } from 'vue'
import { storeToRefs } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'
import type { Task } from '@/model/catalog'
import type { TaskId } from '@/model/branded'

const props = defineProps<{ excluded: ReadonlySet<TaskId>; disabled?: boolean }>()
const emit = defineEmits<{ pick: [taskId: TaskId] }>()

const MAX_MATCHES = 8

const { tasks } = storeToRefs(useConsoleStore())
const query = ref('')
const active = ref(0)
const open = ref(false)
const listId = useId()

/** Finished tasks sink to the bottom: queueing one is allowed, rarely meant. */
const matches = computed<Task[]>(() => {
  const term = query.value.trim().toLowerCase()
  if (!term) return []
  return tasks.value
    .filter((task) => !props.excluded.has(task.id))
    .filter(
      (task) =>
        task.title.toLowerCase().includes(term) ||
        task.label.includes(term) ||
        task.anchor.toLowerCase().includes(term)
    )
    .sort((a, b) => Number(a.status === 'DONE') - Number(b.status === 'DONE'))
    .slice(0, MAX_MATCHES)
})

function workLine(task: Task): string {
  const checklist = task.stepCount - task.draftStepCount
  if (checklist > 0) return `${task.stepsDone} of ${checklist} steps done`
  if (task.reviewState === 'CLAIMED') return 'claimed, waiting for review'
  if (task.reviewState === 'DONE') return 'accepted'
  return 'no checklist'
}

function pick(task: Task | undefined): void {
  if (!task) return
  emit('pick', task.id)
  query.value = ''
  active.value = 0
}

function move(delta: number): void {
  if (matches.value.length === 0) return
  active.value = (active.value + delta + matches.value.length) % matches.value.length
}

function onInput(): void {
  active.value = 0
  open.value = true
}
</script>

<template>
  <div class="relative" data-testid="queue-task-picker">
    <span
      class="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-[15px] leading-none text-accent"
      aria-hidden="true"
      >+</span
    >
    <input
      v-model="query"
      type="text"
      role="combobox"
      autocomplete="off"
      spellcheck="false"
      :disabled="disabled"
      :aria-expanded="open && matches.length > 0"
      :aria-controls="listId"
      :aria-activedescendant="open && matches[active] ? `${listId}-${active}` : undefined"
      aria-label="Add a task to the queue"
      placeholder="Add a task by title or anchor"
      class="field h-(--spacing-control) w-full rounded-[var(--radius-control)] pl-8 pr-3 text-[13px] text-text"
      data-testid="queue-task-input"
      @input="onInput"
      @focus="open = true"
      @blur="open = false"
      @keydown.down.prevent="move(1)"
      @keydown.up.prevent="move(-1)"
      @keydown.enter.prevent="pick(matches[active])"
      @keydown.esc.stop="query = ''"
    />
    <Transition name="popover">
      <ul
        v-if="open && matches.length"
        :id="listId"
        role="listbox"
        class="absolute inset-x-0 bottom-full z-(--z-overlay) mb-1.5 max-h-[300px] overflow-y-auto rounded-[var(--radius-card)] border border-border-strong bg-surface-raised py-1 shadow-modal"
      >
        <li
          v-for="(task, index) in matches"
          :id="`${listId}-${index}`"
          :key="task.id"
          role="option"
          :aria-selected="index === active"
          class="flex cursor-pointer flex-col gap-0.5 px-3 py-1.5"
          :class="index === active ? 'bg-surface-hover' : ''"
          data-testid="queue-task-option"
          @mousedown.prevent="pick(task)"
          @mouseenter="active = index"
        >
          <span class="truncate text-[12.5px] text-text">{{ task.title }}</span>
          <span class="flex items-baseline gap-2 text-[11px]">
            <span class="min-w-0 truncate font-mono text-anchor">{{ task.anchor }}</span>
            <span class="ml-auto shrink-0 text-text-subtle">{{ workLine(task) }}</span>
          </span>
        </li>
      </ul>
    </Transition>
  </div>
</template>
