import { defineStore } from 'pinia'
import { ref } from 'vue'

export type EditorMode = 'write' | 'read'

/** The Write/Read pick last made on any markdown editor; `null` until the reader makes one. */
export const useEditorModeStore = defineStore('editorMode', () => {
  const picked = ref<EditorMode | null>(null)

  function pick(mode: EditorMode): void {
    picked.value = mode
  }

  /** An empty body opens to write, since there is nothing to read; any other opens as last picked. */
  function openingModeFor(body: string, fallback: EditorMode): EditorMode {
    if (!body.trim()) return 'write'
    return picked.value ?? fallback
  }

  return { picked, pick, openingModeFor }
})
