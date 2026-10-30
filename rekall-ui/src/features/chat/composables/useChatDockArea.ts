import { onBeforeUnmount, onMounted, type Ref } from 'vue'
import { useChatDockStore } from '@/features/chat/store/chatDock.store'

/** Tells the dock how wide the room beside the navigator and note list is, as the window resizes it. */
export function useChatDockArea(area: Ref<HTMLElement | null>): void {
  const dock = useChatDockStore()
  let sizeWatcher: ResizeObserver | null = null

  function measure(): void {
    if (area.value) dock.observe(area.value.clientWidth)
  }

  onMounted(() => {
    measure()
    if (typeof ResizeObserver === 'undefined' || !area.value) return
    sizeWatcher = new ResizeObserver(measure)
    sizeWatcher.observe(area.value)
  })
  onBeforeUnmount(() => sizeWatcher?.disconnect())
}
