import { ref, watch, type Ref } from 'vue'
import type { ExposeParam } from 'md-editor-v3'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useEditorSelectionPicker, type EditorSelection } from '@/composables/useEditorSelectionPicker'
import { importPicture } from '@/features/picture/api/pictures.api'
import { alignedAddress } from '@/features/picture/lib/picture-alignment'
import { PICTURE_NOT_INSERTED, pictureMarkdown } from '@/features/picture/lib/picture-file'
import type { PictureAlignment, PictureOwner } from '@/features/picture/model/picture'
import { useToastStore } from '@/stores/toast.store'

/**
 * The picture picker of a markdown editor: the chosen file is copied into the owner's folder, and
 * its markdown replaces the selection the picker was opened over.
 */
export function usePictureInsert(
  editor: Ref<ExposeParam | null>,
  options: { readonly: () => boolean; pathRoot: () => string | null; owner: () => PictureOwner | null }
) {
  const picker = useEditorSelectionPicker(editor, options.readonly)
  const toast = useToastStore()
  const { run } = useAsyncAction()
  /** The folder a picture was last picked in, where the picker opens next. */
  const start = ref<string | null>(null)
  /** How the picture about to be picked is aligned: left each time the picker opens. */
  const alignment = ref<PictureAlignment>('left')
  let folderInView: string | null = null

  watch(options.pathRoot, () => (start.value = null))

  function visit(folder: string): void {
    folderInView = folder
  }

  function toggle(): void {
    if (!picker.open.value) alignment.value = 'left'
    picker.toggle()
  }

  async function copyInto(target: EditorSelection, path: string, owner: PictureOwner, placed: PictureAlignment): Promise<void> {
    const picture = await run(() => importPicture(path, owner))
    if (!picture) return
    if (picker.stillHolds(target)) picker.write(target, pictureMarkdown(path, alignedAddress(picture.url, placed)))
    else toast.notifyError(PICTURE_NOT_INSERTED)
  }

  async function insert(path: string): Promise<void> {
    const target = picker.opened()
    const owner = options.owner()
    start.value = folderInView
    picker.close()
    if (target && owner) await copyInto(target, path, owner, alignment.value)
  }

  return { open: picker.open, start, alignment, toggle, close: picker.close, locateCaret: picker.locateCaret, visit, insert }
}
