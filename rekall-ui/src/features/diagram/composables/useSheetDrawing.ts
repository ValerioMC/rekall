import { computed, ref, shallowRef, watch, type Ref } from 'vue'
import { drawSheet, type SheetDrawing } from '@/features/diagram/lib/sheet-drawing'
import type { DiagramSheet } from '@/features/diagram/model/diagram'

/**
 * Reads and lays out whichever sheet is open. A sheet that changes while an earlier one is still
 * being read wins: the late answer is dropped. A sheet Mermaid cannot read says why.
 */
export function useSheetDrawing(sheet: Ref<DiagramSheet | null>) {
  const drawing = shallowRef<SheetDrawing | null>(null)
  const failure = ref<string | null>(null)
  let generation = 0

  watch(
    sheet,
    async (current) => {
      const mine = ++generation
      drawing.value = null
      failure.value = null
      if (!current) return
      try {
        const drawn = await drawSheet(current)
        if (mine === generation) drawing.value = drawn
      } catch (caught) {
        if (mine === generation) failure.value = caught instanceof Error ? caught.message : String(caught)
      }
    },
    { immediate: true }
  )

  const loading = computed(() => sheet.value !== null && drawing.value === null && failure.value === null)
  return { drawing, failure, loading }
}
