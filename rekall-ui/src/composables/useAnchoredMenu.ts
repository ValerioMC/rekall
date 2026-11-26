import { nextTick, onUnmounted, ref, type Ref } from 'vue'

const EDGE = 8
const GAP = 4

export interface AnchoredMenu {
  open: Ref<boolean>
  menu: Ref<HTMLElement | null>
  placement: Ref<Record<string, string>>
  place: () => void
  show: () => Promise<void>
  toggle: () => Promise<void>
  close: (returnFocus: boolean) => void
  handleKeydown: (event: KeyboardEvent) => void
}

export function useAnchoredMenu(anchorElement: () => HTMLElement | null, width: number): AnchoredMenu {
  const open = ref(false)
  const menu = ref<HTMLElement | null>(null)
  const placement = ref<Record<string, string>>({})

  function place(): void {
    const box = anchorElement()?.getBoundingClientRect()
    if (!box) return
    const menuWidth = Math.min(width, window.innerWidth - EDGE * 2)
    const left = Math.max(EDGE, Math.min(box.right - menuWidth, window.innerWidth - menuWidth - EDGE))
    placement.value = { left: `${left}px`, ...verticalPlacement(box, menu.value), width: `${menuWidth}px` }
  }

  function close(returnFocus: boolean): void {
    open.value = false
    disconnect()
    if (returnFocus) anchorElement()?.focus({ preventScroll: true })
  }

  function closeOnOutside(event: Event): void {
    if (!(event.target instanceof Node)) return close(false)
    if (anchorElement()?.contains(event.target) || menu.value?.contains(event.target)) return
    close(false)
  }

  function disconnect(): void {
    document.removeEventListener('pointerdown', closeOnOutside, true)
    window.removeEventListener('resize', place)
    window.removeEventListener('scroll', place, true)
  }

  async function show(): Promise<void> {
    if (open.value) return
    place()
    open.value = true
    document.addEventListener('pointerdown', closeOnOutside, true)
    window.addEventListener('resize', place)
    window.addEventListener('scroll', place, true)
    await nextTick()
    if (!open.value) return
    place()
    focusMenuStart(menu.value)
  }

  async function toggle(): Promise<void> {
    if (open.value) return close(true)
    await show()
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (event.key === 'Escape' || event.key === 'Tab') {
      event.stopPropagation()
      if (event.key === 'Escape') event.preventDefault()
      return close(true)
    }
    moveMenuFocus(event, menu.value)
  }

  onUnmounted(disconnect)

  return { open, menu, placement, place, show, toggle, close, handleKeydown }
}

function verticalPlacement(box: DOMRect, menu: HTMLElement | null): Record<string, string> {
  const height = menu?.scrollHeight || menu?.offsetHeight || 0
  const below = Math.max(0, window.innerHeight - box.bottom - GAP - EDGE)
  const above = Math.max(0, box.top - GAP - EDGE)
  if (height > below && above > below) {
    return { bottom: `${window.innerHeight - box.top + GAP}px`, maxHeight: `${above}px` }
  }
  return { top: `${box.bottom + GAP}px`, maxHeight: `${below}px` }
}

function menuItems(menu: HTMLElement | null): HTMLElement[] {
  if (!menu) return []
  return [...menu.querySelectorAll<HTMLElement>('[role="menuitem"]')]
    .filter((item) => item.closest('[role="menu"]') === menu && !item.closest('[hidden]'))
}

function focusMenuStart(menu: HTMLElement | null): void {
  const items = menuItems(menu)
  const initial = items.find((item) => item.hasAttribute('data-menu-initial'))
  const firstAvailable = items.find((item) => item.getAttribute('aria-disabled') !== 'true')
  const first = initial ?? firstAvailable ?? items[0]
  focusMenuItem(menu, first)
}

function moveMenuFocus(event: KeyboardEvent, menu: HTMLElement | null): void {
  const step = event.key === 'ArrowDown' ? 1 : event.key === 'ArrowUp' ? -1 : 0
  if (step === 0) return
  event.preventDefault()
  event.stopPropagation()
  const items = menuItems(menu)
  const at = items.indexOf(document.activeElement as HTMLElement)
  focusMenuItem(menu, items[(at + step + items.length) % items.length])
}

function focusMenuItem(menu: HTMLElement | null, item: HTMLElement | undefined): void {
  if (!menu || !item) return
  item.focus({ preventScroll: true })
  const bounds = menu.getBoundingClientRect()
  const row = item.getBoundingClientRect()
  if (row.top < bounds.top) menu.scrollTop += row.top - bounds.top
  if (row.bottom > bounds.bottom) menu.scrollTop += row.bottom - bounds.bottom
}
