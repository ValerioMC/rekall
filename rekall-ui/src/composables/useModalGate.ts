import { ref } from 'vue'

const openCount = ref(0)

export function useModalGate() {
  return {
    isModalOpen: openCount,
    open: (): void => {
      openCount.value += 1
    },
    close: (): void => {
      openCount.value = Math.max(0, openCount.value - 1)
    }
  }
}
