<script setup lang="ts">
import { computed, onMounted, ref, useId } from 'vue'
import { storeToRefs } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'
import { TASK_STATUS_LABEL, isEpic, type Task } from '@/features/catalog/model/catalog'
import type { TaskId } from '@/model/branded'

/**
 * Files a task that already exists under the epic, found by title, label or anchor. Only tasks of
 * the epic's project are offered; one in another epic is offered too, says so, and moves here.
 */
const props = defineProps<{ epic: Task }>()

const emit = defineEmits<{ pick: [taskId: TaskId]; close: [] }>()

const MAX_MATCHES = 8

const { tasks } = storeToRefs(useConsoleStore())
const query = ref('')
const active = ref(0)
const field = ref<HTMLInputElement | null>(null)
const listId = useId()

const candidates = computed<Task[]>(() =>
  tasks.value.filter(
    (task) => task.projectId === props.epic.projectId && !isEpic(task) && task.epicId !== props.epic.id
  )
)

/** Live work first, finished tasks last, by title within each. */
const matches = computed<Task[]>(() => {
  const term = query.value.trim().toLowerCase()
  return candidates.value
    .filter(
      (task) =>
        !term ||
        task.title.toLowerCase().includes(term) ||
        task.label.includes(term) ||
        task.anchor.toLowerCase().includes(term)
    )
    .sort(
      (left, right) =>
        Number(left.status === 'DONE') - Number(right.status === 'DONE') || left.title.localeCompare(right.title)
    )
    .slice(0, MAX_MATCHES)
})

function whereItIs(task: Task): string {
  const current = task.epicId ? tasks.value.find((candidate) => candidate.id === task.epicId) : null
  return current ? `in ${current.title}` : TASK_STATUS_LABEL[task.status]
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

onMounted(() => field.value?.focus())
</script>

<template>
  <div class="relative" data-testid="epic-task-picker">
    <input
      ref="field"
      v-model="query"
      type="text"
      role="combobox"
      autocomplete="off"
      spellcheck="false"
      :aria-expanded="matches.length > 0"
      :aria-controls="listId"
      :aria-activedescendant="matches[active] ? `${listId}-${active}` : undefined"
      aria-label="Find a task to add to the epic"
      placeholder="Find a task by title or anchor"
      class="field h-(--spacing-control) w-full rounded-control px-3 text-body text-text"
      data-testid="epic-task-picker-input"
      @input="active = 0"
      @keydown.down.prevent="move(1)"
      @keydown.up.prevent="move(-1)"
      @keydown.enter.prevent="pick(matches[active])"
      @keydown.esc.stop="emit('close')"
    />
    <ul
      v-if="matches.length"
      :id="listId"
      role="listbox"
      class="mt-1.5 max-h-[300px] overflow-y-auto rounded-control border border-border bg-surface py-1"
    >
      <li
        v-for="(task, index) in matches"
        :id="`${listId}-${index}`"
        :key="task.id"
        role="option"
        :aria-selected="index === active"
        class="flex cursor-pointer flex-col gap-0.5 px-3 py-1.5"
        :class="index === active ? 'bg-surface-hover' : ''"
        data-testid="epic-task-option"
        @mousedown.prevent="pick(task)"
        @mouseenter="active = index"
      >
        <span class="truncate text-small text-text">{{ task.title }}</span>
        <span class="flex items-baseline gap-2 text-caption">
          <span class="min-w-0 truncate font-mono text-anchor">{{ task.label }}</span>
          <span class="ml-auto shrink-0 text-text-subtle">{{ whereItIs(task) }}</span>
        </span>
      </li>
    </ul>
    <p v-else class="mt-1.5 px-1 text-caption text-text-subtle" data-testid="epic-task-picker-empty">
      {{ candidates.length ? 'No task matches that.' : 'Every task of this project is already in the epic.' }}
    </p>
  </div>
</template>
