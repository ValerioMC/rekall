<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { enterDialog, leaveDialog } from '@/common/a11y/dialog-stack'
import { TASK_STATUS_COLOR, TASK_STATUS_LABEL, type Task } from '@/features/catalog/model/catalog'
import type { TaskId } from '@/model/branded'

/**
 * The tasks a task can wait for, hung from the button that opened it: type to narrow, pick one
 * and it closes. It opens above a modal and takes the keyboard from it, so Escape closes the
 * panel alone.
 */
const props = defineProps<{
  tasks: readonly Task[]
  /** The element the panel hangs from: its left edge is the panel's left edge. */
  anchor: HTMLElement
}>()

const emit = defineEmits<{ pick: [taskId: TaskId]; close: [] }>()

const WIDTH = 340
const GUTTER = 8
const PANEL_HEIGHT = 340

const panel = ref<HTMLElement | null>(null)
const filterField = ref<HTMLInputElement | null>(null)
const filter = ref('')
const highlighted = ref(0)
const position = ref({ top: 0, left: 0, width: 0, opensUp: false })
let dialogToken: symbol | null = null

const shown = computed(() => {
  const needle = filter.value.trim().toLowerCase()
  return props.tasks.filter((task) => task.title.toLowerCase().includes(needle))
})

const emptyText = computed(() => (props.tasks.length ? 'No task matches that.' : 'No other task left to add.'))

watch(filter, () => (highlighted.value = 0))

function pick(task: Task): void {
  emit('pick', task.id)
}

function place(): void {
  const rect = props.anchor.getBoundingClientRect()
  const width = Math.min(WIDTH, window.innerWidth - 2 * GUTTER)
  const left = Math.max(GUTTER, Math.min(rect.left, window.innerWidth - width - GUTTER))
  const opensUp = window.innerHeight - rect.bottom < PANEL_HEIGHT && rect.top > window.innerHeight - rect.bottom
  position.value = { top: opensUp ? window.innerHeight - rect.top + 6 : rect.bottom + 6, left, width, opensUp }
}

function moveHighlight(delta: number): void {
  if (!shown.value.length) return
  highlighted.value = (highlighted.value + delta + shown.value.length) % shown.value.length
  panel.value?.querySelector<HTMLElement>(`[data-row-index="${highlighted.value}"]`)?.scrollIntoView({ block: 'nearest' })
}

function onKeydown(event: KeyboardEvent): void {
  event.stopPropagation()
  if (event.key === 'Escape') return emit('close')
  if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
    event.preventDefault()
    return moveHighlight(event.key === 'ArrowDown' ? 1 : -1)
  }
  if (event.key !== 'Enter') return
  event.preventDefault()
  const task = shown.value[highlighted.value]
  if (task) pick(task)
}

function onPointerDown(event: PointerEvent): void {
  const target = event.target as Node | null
  if (!target || panel.value?.contains(target) || props.anchor.contains(target)) return
  emit('close')
}

onMounted(async () => {
  dialogToken = enterDialog()
  place()
  window.addEventListener('keydown', onKeydown, true)
  window.addEventListener('pointerdown', onPointerDown, true)
  window.addEventListener('resize', place)
  await nextTick()
  filterField.value?.focus()
})

onUnmounted(() => {
  if (dialogToken !== null) leaveDialog(dialogToken)
  window.removeEventListener('keydown', onKeydown, true)
  window.removeEventListener('pointerdown', onPointerDown, true)
  window.removeEventListener('resize', place)
})
</script>

<template>
  <Teleport to="body">
    <div
      ref="panel"
      class="fixed z-(--z-popover) flex max-h-[min(340px,calc(100vh-24px))] flex-col overflow-hidden rounded-card border border-border-strong bg-surface shadow-lift"
      :style="{
        left: `${position.left}px`,
        width: `${position.width}px`,
        ...(position.opensUp ? { bottom: `${position.top}px` } : { top: `${position.top}px` })
      }"
      role="dialog"
      aria-label="Tasks this one can depend on"
      data-testid="task-dependency-picker"
    >
      <div class="border-b border-border px-3 py-2">
        <label class="sr-only" for="task-dependency-picker-filter">Find a task</label>
        <input
          id="task-dependency-picker-filter"
          ref="filterField"
          v-model="filter"
          type="search"
          autocomplete="off"
          spellcheck="false"
          placeholder="Find a task"
          class="focus-ring h-7 w-full rounded-control border border-border bg-canvas px-2.5 text-small text-text placeholder:text-text-subtle"
          data-testid="task-dependency-picker-filter"
        />
      </div>

      <div class="min-h-0 flex-1 overflow-y-auto py-1">
        <p
          v-if="!shown.length"
          class="px-3.5 py-3 text-caption text-text-subtle"
          data-testid="task-dependency-picker-empty"
        >
          {{ emptyText }}
        </p>
        <div v-else role="listbox" aria-label="Tasks this one can depend on">
          <button
            v-for="(task, index) in shown"
            :key="task.id"
            type="button"
            role="option"
            class="focus-ring flex w-full items-center gap-2.5 px-3.5 py-1.5 text-left transition-colors"
            :class="index === highlighted ? 'bg-surface-raised' : 'hover:bg-surface-raised'"
            :aria-selected="index === highlighted"
            :data-row-index="index"
            data-testid="task-dependency-picker-row"
            @mousemove="highlighted = index"
            @click="pick(task)"
          >
            <span class="size-2 shrink-0 rounded-full" :class="TASK_STATUS_COLOR[task.status]" aria-hidden="true" />
            <span class="min-w-0 flex-1 truncate text-small text-text">{{ task.title }}</span>
            <span class="shrink-0 text-micro text-text-subtle">{{ TASK_STATUS_LABEL[task.status] }}</span>
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
