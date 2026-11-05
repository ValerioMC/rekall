import { onBeforeUnmount, watch, type Ref } from 'vue'

/** Calls `dismiss` for a press anywhere outside `host`, listening only while `active` is true. */
export function useDismissOnOutsidePress(host: Ref<HTMLElement | null>, active: Ref<boolean>, dismiss: () => void): void {
  function onPointerDown(event: PointerEvent): void {
    if (host.value && event.target instanceof Node && !host.value.contains(event.target)) dismiss()
  }

  function stopListening(): void {
    window.removeEventListener('pointerdown', onPointerDown, true)
  }

  watch(active, (isActive) => {
    if (isActive) window.addEventListener('pointerdown', onPointerDown, true)
    else stopListening()
  })

  onBeforeUnmount(stopListening)
}
