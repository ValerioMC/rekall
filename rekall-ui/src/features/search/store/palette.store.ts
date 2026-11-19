import { defineStore } from 'pinia'
import { ref } from 'vue'

/** Whether the command palette is on screen: ⌘K, the header bar and the menu all open the same one. */
export const usePaletteStore = defineStore('palette', () => {
  const isOpen = ref(false)

  function show(): void {
    isOpen.value = true
  }

  function hide(): void {
    isOpen.value = false
  }

  function toggle(): void {
    isOpen.value = !isOpen.value
  }

  return { isOpen, show, hide, toggle }
})
