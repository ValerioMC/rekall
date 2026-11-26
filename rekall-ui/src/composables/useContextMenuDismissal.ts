import { onMounted, onUnmounted, type Ref } from 'vue'

/** A menu opened at the pointer closes on a press outside it, on Escape and when the window loses focus. */
export function useContextMenuDismissal(menu: Ref<HTMLElement | null>, close: () => void): void {
  function onMousedown(event: MouseEvent): void {
    if (!(event.target instanceof Node) || !menu.value?.contains(event.target)) close()
  }

  function onKeydown(event: KeyboardEvent): void {
    if (event.key === 'Escape') close()
  }

  onMounted(() => {
    document.addEventListener('mousedown', onMousedown, true)
    document.addEventListener('keydown', onKeydown)
    window.addEventListener('blur', close)
  })

  onUnmounted(() => {
    document.removeEventListener('mousedown', onMousedown, true)
    document.removeEventListener('keydown', onKeydown)
    window.removeEventListener('blur', close)
  })
}
