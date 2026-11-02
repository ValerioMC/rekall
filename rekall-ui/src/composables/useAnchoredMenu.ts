import { nextTick, onUnmounted, ref, type Ref } from 'vue'

const EDGE = 8

export interface AnchoredMenu {
  open: Ref<boolean>
  menu: Ref<HTMLElement | null>
  placement: Ref<Record<string, string>>
  toggle: () => Promise<void>
  close: (returnFocus: boolean) => void
  handleKeydown: (event: KeyboardEvent) => void
}

/**
 * The behaviour of a menu that hangs under a key: placed against the key's right edge inside the
 * viewport, closed by an outside press, Escape or Tab, and walked with the arrow keys. The caller
 * renders the menu with `role="menuitem"` buttons and binds `menu`, `placement` and `handleKeydown`.
 */
export function useAnchoredMenu(anchorElement: () => HTMLElement | null, width: number): AnchoredMenu {
  const open = ref(false)
  const menu = ref<HTMLElement | null>(null)
  const placement = ref<Record<string, string>>({})

  function place(): void {
    const box = anchorElement()?.getBoundingClientRect()
    if (!box) return
    const left = Math.max(EDGE, Math.min(box.right - width, window.innerWidth - width - EDGE))
    placement.value = { left: `${left}px`, top: `${box.bottom + 4}px`, width: `${width}px` }
  }

  function menuItems(): HTMLElement[] {
    return [...(menu.value?.querySelectorAll<HTMLElement>('[role="menuitem"]') ?? [])]
  }

  function close(returnFocus: boolean): void {
    open.value = false
    document.removeEventListener('pointerdown', closeOnOutside, true)
    if (returnFocus) anchorElement()?.focus()
  }

  function closeOnOutside(event: Event): void {
    if (!(event.target instanceof Node)) return close(false)
    if (anchorElement()?.contains(event.target) || menu.value?.contains(event.target)) return
    close(false)
  }

  async function toggle(): Promise<void> {
    if (open.value) return close(true)
    place()
    open.value = true
    document.addEventListener('pointerdown', closeOnOutside, true)
    await nextTick()
    menuItems()[0]?.focus()
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (event.key === 'Escape' || event.key === 'Tab') {
      event.stopPropagation()
      return close(event.key === 'Escape')
    }
    const step = event.key === 'ArrowDown' ? 1 : event.key === 'ArrowUp' ? -1 : 0
    if (step === 0) return
    event.preventDefault()
    const items = menuItems()
    const at = items.indexOf(document.activeElement as HTMLElement)
    items[(at + step + items.length) % items.length]?.focus()
  }

  onUnmounted(() => document.removeEventListener('pointerdown', closeOnOutside, true))

  return { open, menu, placement, toggle, close, handleKeydown }
}
