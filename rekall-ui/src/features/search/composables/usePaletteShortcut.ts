import { onMounted, onUnmounted } from 'vue'
import { useModalGate } from '@/composables/useModalGate'
import { usePaletteStore } from '@/features/search/store/palette.store'

/** ⌘K or Ctrl+K opens the palette on every screen, and closes it again; it never opens over another dialog. */
export function usePaletteShortcut(): void {
  const palette = usePaletteStore()
  const { isModalOpen } = useModalGate()

  function onKeydown(event: KeyboardEvent): void {
    if (!(event.metaKey || event.ctrlKey) || event.key.toLowerCase() !== 'k') return
    event.preventDefault()
    if (palette.isOpen) palette.hide()
    else if (isModalOpen.value === 0) palette.show()
  }

  onMounted(() => window.addEventListener('keydown', onKeydown))
  onUnmounted(() => window.removeEventListener('keydown', onKeydown))
}
