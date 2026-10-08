<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import {
  TABLE_MENU_GROUPS,
  isTableActionAvailable,
  type TableMenuAction
} from '@/common/live-preview/table-menu'
import type { TableMenuRequest } from '@/common/live-preview/table-widget'

/** The right-click menu over a drawn table: rows, columns, alignment, and its markdown. */
const props = defineProps<{ request: TableMenuRequest }>()

const emit = defineEmits<{ choose: [action: TableMenuAction]; close: [] }>()

const menu = ref<HTMLElement | null>(null)

// Kept inside the window: a menu opened near the right or bottom edge flips back over the click.
const MENU_WIDTH = 200
const MENU_HEIGHT = 330
const left = Math.min(props.request.x, window.innerWidth - MENU_WIDTH - 8)
const top = Math.min(props.request.y, window.innerHeight - MENU_HEIGHT - 8)

function available(action: TableMenuAction): boolean {
  return isTableActionAvailable(action, props.request.cell, props.request.columnCount)
}

function closeOnOutside(event: MouseEvent): void {
  if (!(event.target instanceof Node) || !menu.value?.contains(event.target)) emit('close')
}

function closeOnEscape(event: KeyboardEvent): void {
  if (event.key === 'Escape') emit('close')
}

function closeOnWindowBlur(): void {
  emit('close')
}

onMounted(() => {
  document.addEventListener('mousedown', closeOnOutside, true)
  document.addEventListener('keydown', closeOnEscape)
  window.addEventListener('blur', closeOnWindowBlur)
  menu.value?.querySelector<HTMLElement>('button:not(:disabled)')?.focus()
})

onUnmounted(() => {
  document.removeEventListener('mousedown', closeOnOutside, true)
  document.removeEventListener('keydown', closeOnEscape)
  window.removeEventListener('blur', closeOnWindowBlur)
})
</script>

<template>
  <div
    ref="menu"
    role="menu"
    aria-label="Table"
    class="fixed z-[var(--z-overlay)] w-[200px] rounded-card border border-border bg-surface-raised p-1 shadow-lg"
    :style="{ left: `${left}px`, top: `${top}px` }"
    data-testid="table-context-menu"
  >
    <template v-for="(group, index) in TABLE_MENU_GROUPS" :key="index">
      <div v-if="index > 0" class="mx-1 my-1 h-px bg-border" role="separator" />
      <button
        v-for="item in group"
        :key="item.action"
        type="button"
        role="menuitem"
        class="focus-ring flex h-7 w-full items-center rounded-control px-2 text-left text-small text-text-muted transition-colors enabled:hover:bg-surface-hover enabled:hover:text-text disabled:opacity-40"
        :disabled="!available(item.action)"
        :data-testid="`table-menu-${item.action}`"
        @click="emit('choose', item.action)"
      >
        {{ item.label }}
      </button>
    </template>
  </div>
</template>
