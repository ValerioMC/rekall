<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'
import TagIcon from '@/components/ui/TagIcon.vue'
import { enterDialog, leaveDialog } from '@/common/a11y/dialog-stack'
import type { TagId } from '@/model/branded'
import type { Tag } from '@/features/catalog/model/catalog'

/**
 * The configured tags hung from the button that opened them, to pick the one a task carries or
 * none. A pick is sent up at once and the panel closes. It takes the keyboard from a modal under it.
 */
const props = defineProps<{
  tags: readonly Tag[]
  selected: TagId | null
  /** The element the panel hangs from: its left edge is the panel's left edge. */
  anchor: HTMLElement
}>()

const emit = defineEmits<{ select: [tagId: TagId | null]; close: [] }>()

const WIDTH = 240
const GUTTER = 8
const PANEL_HEIGHT = 320

const panel = ref<HTMLElement | null>(null)
const highlighted = ref(0)
const position = ref({ top: 0, left: 0, width: WIDTH, opensUp: false })
let dialogToken: symbol | null = null

/** "No tag" leads, so clearing the tag is always one row away. */
const rows = computed<readonly (Tag | null)[]>(() => [null, ...props.tags])

function isChosen(row: Tag | null): boolean {
  return (row?.id ?? null) === props.selected
}

function choose(row: Tag | null): void {
  emit('select', row?.id ?? null)
}

function place(): void {
  const rect = props.anchor.getBoundingClientRect()
  const width = Math.min(WIDTH, window.innerWidth - 2 * GUTTER)
  const opensUp =
    window.innerHeight - rect.bottom < PANEL_HEIGHT && rect.top > window.innerHeight - rect.bottom
  position.value = {
    top: opensUp ? window.innerHeight - rect.top + 6 : rect.bottom + 6,
    left: Math.max(GUTTER, Math.min(rect.left, window.innerWidth - width - GUTTER)),
    width,
    opensUp
  }
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    event.stopPropagation()
    emit('close')
    return
  }
  if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
    event.preventDefault()
    event.stopPropagation()
    const delta = event.key === 'ArrowDown' ? 1 : -1
    highlighted.value = (highlighted.value + delta + rows.value.length) % rows.value.length
    return
  }
  if (event.key === 'Enter') {
    event.preventDefault()
    event.stopPropagation()
    choose(rows.value[highlighted.value] ?? null)
  }
}

function onPointerDown(event: PointerEvent): void {
  const target = event.target as Node | null
  if (!target || panel.value?.contains(target) || props.anchor.contains(target)) return
  emit('close')
}

onMounted(async () => {
  dialogToken = enterDialog()
  place()
  highlighted.value = Math.max(0, rows.value.findIndex(isChosen))
  window.addEventListener('keydown', onKeydown, true)
  window.addEventListener('pointerdown', onPointerDown, true)
  window.addEventListener('resize', place)
  await nextTick()
  panel.value?.focus()
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
      tabindex="-1"
      class="fixed z-(--z-popover) overflow-hidden rounded-card border border-border-strong bg-surface shadow-lift outline-none"
      :style="{
        left: `${position.left}px`,
        width: `${position.width}px`,
        ...(position.opensUp ? { bottom: `${position.top}px` } : { top: `${position.top}px` })
      }"
      role="dialog"
      aria-label="Tag for this task"
      data-testid="tag-picker"
    >
      <div role="listbox" aria-label="Tags" class="max-h-[min(280px,calc(100vh-24px))] overflow-y-auto py-1">
        <button
          v-for="(row, index) in rows"
          :key="row?.id ?? 'none'"
          type="button"
          role="option"
          class="focus-ring flex h-8 w-full cursor-pointer items-center gap-2 px-3 text-left text-small transition-colors duration-(--motion-quick)"
          :class="index === highlighted ? 'bg-surface-raised' : 'hover:bg-surface-raised'"
          :aria-selected="isChosen(row)"
          data-testid="tag-picker-row"
          @mousemove="highlighted = index"
          @click="choose(row)"
        >
          <TagIcon v-if="row" :icon="row.icon" :color="row.color" :size="14" />
          <span v-else class="size-3.5 shrink-0" aria-hidden="true" />
          <span class="min-w-0 flex-1 truncate" :class="row ? 'font-medium text-text' : 'text-text-muted'">
            {{ row ? row.name : 'No tag' }}
          </span>
          <svg
            v-if="isChosen(row)"
            class="size-3 shrink-0 text-anchor"
            viewBox="0 0 10 10"
            fill="none"
            aria-hidden="true"
          >
            <path
              d="M2 5.2l2 2 4-4.4"
              stroke="currentColor"
              stroke-width="1.6"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </button>
      </div>
      <p
        v-if="!tags.length"
        class="border-t border-border px-3 py-2 text-caption text-text-subtle"
        data-testid="tag-picker-empty"
      >
        No tags configured yet. Add one from the tags button in the header.
      </p>
    </div>
  </Teleport>
</template>
