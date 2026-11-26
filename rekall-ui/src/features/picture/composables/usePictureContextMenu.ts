import { ref, type Ref } from 'vue'
import type { ExposeParam } from 'md-editor-v3'
import { rewritePictureAddress } from '@/common/live-preview/image-edit'
import type { PictureMenuRequest } from '@/common/live-preview/image-widget'
import { alignedAddress } from '@/features/picture/lib/picture-alignment'
import type { PictureAlignment } from '@/features/picture/model/picture'

/** The menu a right-click on a drawn picture opens, and the edit its choice makes in the editor. */
export function usePictureContextMenu(editor: Ref<ExposeParam | null>): {
  request: Ref<PictureMenuRequest | null>
  open: (event: Event) => void
  choose: (alignment: PictureAlignment) => void
  close: () => void
} {
  const request = ref<PictureMenuRequest | null>(null)

  function open(event: Event): void {
    if (event instanceof CustomEvent) request.value = event.detail as PictureMenuRequest
  }

  function choose(alignment: PictureAlignment): void {
    const target = request.value
    const view = editor.value?.getEditorView()
    request.value = null
    if (!target || !view) return
    rewritePictureAddress(view, target.picturePosition, (address) => alignedAddress(address, alignment))
  }

  return { request, open, choose, close: () => (request.value = null) }
}
