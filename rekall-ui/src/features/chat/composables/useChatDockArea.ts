import { onBeforeUnmount, onMounted, watch, type Ref } from 'vue'
import { useChatDockStore } from '@/features/chat/store/chatDock.store'

/**
 * Tells the dock how wide the room it docks in is, as the window resizes it. `inline` says the
 * area has the note pane as a sibling to share with; `enabled` lets a layout that is not on
 * screen leave the dock to the one that is.
 */
export function useChatDockArea(
  area: Ref<HTMLElement | null>,
  inline: boolean,
  enabled: () => boolean = () => true
): void {
  const dock = useChatDockStore()
  let sizeWatcher: ResizeObserver | null = null

  function measure(): void {
    if (area.value && enabled()) dock.observe(area.value.clientWidth, inline)
  }

  onMounted(() => {
    measure()
    if (typeof ResizeObserver === 'undefined' || !area.value) return
    sizeWatcher = new ResizeObserver(measure)
    sizeWatcher.observe(area.value)
  })
  watch(enabled, measure)
  onBeforeUnmount(() => sizeWatcher?.disconnect())
}
