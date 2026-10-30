import { nextTick, ref, watch, type Ref } from 'vue'

const STICK_DISTANCE_PX = 48

export interface StickToBottom {
  stuck: Ref<boolean>
  unseen: Ref<number>
  onScroll: () => void
  jumpToLatest: () => void
}

/**
 * Keeps a scrolling log at its bottom while the reader is within 48 px of it. Scrolling up lets go
 * and counts the items that arrive meanwhile, until the reader jumps back or scrolls down.
 */
export function useStickToBottom(
  scroller: Ref<HTMLElement | null>,
  items: () => readonly unknown[]
): StickToBottom {
  const stuck = ref(true)
  const unseen = ref(0)

  function distanceFromBottom(element: HTMLElement): number {
    return element.scrollHeight - element.scrollTop - element.clientHeight
  }

  function scrollToBottom(element: HTMLElement): void {
    element.scrollTop = element.scrollHeight
  }

  function onScroll(): void {
    const element = scroller.value
    if (!element) return
    stuck.value = distanceFromBottom(element) <= STICK_DISTANCE_PX
    if (stuck.value) unseen.value = 0
  }

  function jumpToLatest(): void {
    const element = scroller.value
    if (element) scrollToBottom(element)
    stuck.value = true
    unseen.value = 0
  }

  let knownCount = items().length

  watch(items, async (current) => {
    const added = Math.max(0, current.length - knownCount)
    knownCount = current.length
    if (!stuck.value) {
      unseen.value += added
      return
    }
    await nextTick()
    if (scroller.value) scrollToBottom(scroller.value)
  })

  return { stuck, unseen, onScroll, jumpToLatest }
}
