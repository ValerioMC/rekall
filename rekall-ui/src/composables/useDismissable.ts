import { onBeforeUnmount, watch, type Ref } from 'vue'

/** Closes `open` when the pointer goes down outside `host` or Escape is pressed, for as long as it is open. */
export function useDismissable(host: Ref<HTMLElement | null>, open: Ref<boolean>): void {
  function onPointerDown(event: PointerEvent): void {
    if (host.value && event.target instanceof Node && !host.value.contains(event.target)) open.value = false
  }

  function onKeydown(event: KeyboardEvent): void {
    if (event.key === 'Escape') open.value = false
  }

  function stop(): void {
    window.removeEventListener('pointerdown', onPointerDown, true)
    window.removeEventListener('keydown', onKeydown)
  }

  watch(open, (isOpen) => {
    stop()
    if (!isOpen) return
    window.addEventListener('pointerdown', onPointerDown, true)
    window.addEventListener('keydown', onKeydown)
  })
  onBeforeUnmount(stop)
}
