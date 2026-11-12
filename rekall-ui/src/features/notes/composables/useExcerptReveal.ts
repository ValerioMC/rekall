import { onBeforeUnmount, ref, type Ref } from 'vue'
import type { DocumentId } from '@/model/branded'

const REVEAL_DELAY_MS = 350

export interface ExcerptReveal {
  readonly revealedId: Ref<DocumentId | null>
  reveal(id: DocumentId): void
  hide(): void
}

/** Which note's excerpt is on screen: one at a time, shown only after the pointer or focus has stayed on its row. */
export function useExcerptReveal(): ExcerptReveal {
  const revealedId = ref<DocumentId | null>(null)
  let pending: ReturnType<typeof setTimeout> | null = null

  function cancelPending(): void {
    if (pending) clearTimeout(pending)
    pending = null
  }

  function reveal(id: DocumentId): void {
    cancelPending()
    pending = setTimeout(() => {
      revealedId.value = id
    }, REVEAL_DELAY_MS)
  }

  function hide(): void {
    cancelPending()
    revealedId.value = null
  }

  onBeforeUnmount(cancelPending)

  return { revealedId, reveal, hide }
}
