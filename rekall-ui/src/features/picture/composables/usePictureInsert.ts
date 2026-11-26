import { ref, watch, type Ref } from 'vue'
import type { ExposeParam } from 'md-editor-v3'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useEditorSelectionPicker, type EditorSelection } from '@/composables/useEditorSelectionPicker'
import { importPicture } from '@/features/picture/api/pictures.api'
import { PICTURE_NOT_INSERTED, pictureMarkdown } from '@/features/picture/lib/picture-file'
import type { PictureOwner } from '@/features/picture/model/picture'
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
  let folderInView: string | null = null

  watch(options.pathRoot, () => (start.value = null))

  function visit(folder: string): void {
    folderInView = folder
  }

  async function copyInto(target: EditorSelection, path: string, owner: PictureOwner): Promise<void> {
    const picture = await run(() => importPicture(path, owner))
    if (!picture) return
    if (picker.stillHolds(target)) picker.write(target, pictureMarkdown(path, picture.url))
    else toast.notifyError(PICTURE_NOT_INSERTED)
  }

  async function insert(path: string): Promise<void> {
    const target = picker.opened()
    const owner = options.owner()
    start.value = folderInView
    picker.close()
    if (target && owner) await copyInto(target, path, owner)
  }

  return { open: picker.open, start, toggle: picker.toggle, close: picker.close, locateCaret: picker.locateCaret, visit, insert }
}
