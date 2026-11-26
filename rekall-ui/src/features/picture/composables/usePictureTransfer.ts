import { nextTick, onMounted, watch, type Ref } from 'vue'
import type { ExposeParam } from 'md-editor-v3'
import type { EditorView } from '@codemirror/view'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { uploadPicture } from '@/features/picture/api/pictures.api'
import { fillPlace, holdPlace, relabelPlace, type TextRange } from '@/features/picture/lib/held-places'
import { PASTED_PICTURE_NAME, PICTURE_NOT_INSERTED, pictureMarkdown } from '@/features/picture/lib/picture-file'
import { sendPictureReceiver, type PictureReceiver } from '@/features/picture/lib/picture-transfer'
import type { PictureOwner, TransferredPicture } from '@/features/picture/model/picture'
import { useToastStore } from '@/stores/toast.store'

function copyingLabel(pictures: readonly TransferredPicture[], index: number): string {
  if (pictures.length > 1) return `Copying picture ${index + 1} of ${pictures.length}`
  return `Copying ${pictures[0]?.name ?? 'pasted picture'}`
}

/**
 * Pictures pasted or dropped on a markdown editor are copied into the owner's folder, and their
 * markdown is written, in the order they came, where they arrived.
 */
export function usePictureTransfer(editor: Ref<ExposeParam | null>, owner: () => PictureOwner | null): void {
  const toast = useToastStore()
  const { run } = useAsyncAction()

  function view(): EditorView | null {
    return editor.value?.getEditorView() ?? null
  }

  async function copy(picture: TransferredPicture, into: PictureOwner): Promise<string | null> {
    const kept = await run(() => uploadPicture(picture.file, picture.name, into))
    return kept && pictureMarkdown(picture.name ?? PASTED_PICTURE_NAME, kept.url)
  }

  async function copyInto(
    target: EditorView,
    pictures: readonly TransferredPicture[],
    range: TextRange,
    into: PictureOwner
  ): Promise<void> {
    const place = holdPlace(target, range, copyingLabel(pictures, 0))
    const written: string[] = []
    for (const [index, picture] of pictures.entries()) {
      if (view() !== target || !relabelPlace(target, place, copyingLabel(pictures, index))) break
      const markdown = await copy(picture, into)
      if (markdown) written.push(markdown)
    }
    const filled = view() === target && fillPlace(target, place, written.join('\n\n'))
    if (!filled && written.length > 0) toast.notifyError(PICTURE_NOT_INSERTED)
  }

  function receiverFor(into: PictureOwner): PictureReceiver {
    return (target, pictures, range) => void copyInto(target, pictures, range, into)
  }

  function send(): void {
    void nextTick(() => {
      const current = view()
      const into = owner()
      if (current) sendPictureReceiver(current, into && receiverFor(into))
    })
  }

  onMounted(send)
  watch([() => editor.value, () => owner()?.kind, () => owner()?.id], send)
}
