<script setup lang="ts">
import { nextTick, ref } from 'vue'
import KindGlyph from '@/features/diagram/components/canvas/KindGlyph.vue'
import { KIND_INFO } from '@/features/diagram/lib/kinds'
import type { DiagramSheet } from '@/features/diagram/model/diagram'

/**
 * One tab per sheet of the diagram. Arrow keys move between them, as in any tab list, and the
 * sheet changes as focus does.
 */
const props = defineProps<{ sheets: readonly DiagramSheet[]; active: number }>()

const emit = defineEmits<{ select: [index: number] }>()

const tabs = ref<HTMLButtonElement[]>([])

async function move(by: number): Promise<void> {
  const next = (props.active + by + props.sheets.length) % props.sheets.length
  emit('select', next)
  await nextTick()
  tabs.value[next]?.focus()
}

function onKeydown(event: KeyboardEvent): void {
  const steps: Record<string, number> = { ArrowRight: 1, ArrowLeft: -1 }
  const by = steps[event.key]
  if (by === undefined) return
  event.preventDefault()
  void move(by)
}

function labelOf(sheet: DiagramSheet): string {
  return sheet.title ?? KIND_INFO[sheet.kind].label
}
</script>

<template>
  <div class="glass flex h-10 items-center gap-0.5 rounded-card border border-border p-1 shadow-lift" role="tablist" aria-label="Sheets" @keydown="onKeydown">
    <button
      v-for="(sheet, index) in sheets"
      :id="`sheet-tab-${index}`"
      :key="index"
      ref="tabs"
      role="tab"
      :aria-selected="index === active"
      aria-controls="diagram-sheet"
      :tabindex="index === active ? 0 : -1"
      class="focus-ring flex h-8 items-center gap-1.5 rounded-control px-2.5 text-small transition-colors"
      :class="index === active ? 'bg-surface-raised text-text shadow-[0_1px_2px_rgb(0_0_0/0.4)]' : 'text-text-subtle hover:text-text'"
      :title="KIND_INFO[sheet.kind].purpose"
      data-testid="sheet-tab"
      @click="emit('select', index)"
    >
      <KindGlyph :kind="sheet.kind" :size="14" />
      <span class="max-w-[160px] truncate">{{ labelOf(sheet) }}</span>
    </button>
  </div>
</template>
