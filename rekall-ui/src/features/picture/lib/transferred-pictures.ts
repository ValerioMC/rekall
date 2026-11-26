import { isPictureFileName } from '@/features/picture/lib/picture-file'
import type { TransferredPicture } from '@/features/picture/model/picture'

/** The name browsers give a screenshot or a copied picture, which is no name of its own. */
const CLIPBOARD_PLACEHOLDER_NAME = 'image.png'

/** Any `image/` type counts: the server judges the content and says why it refuses one. */
function pictureFilesOf(transfer: DataTransfer | null): File[] {
  return Array.from(transfer?.files ?? []).filter((file) => file.type.startsWith('image/') || isPictureFileName(file.name))
}

export function pastedPictures(clipboard: DataTransfer | null): TransferredPicture[] {
  return pictureFilesOf(clipboard).map((file) => ({
    file,
    name: file.name === CLIPBOARD_PLACEHOLDER_NAME ? null : file.name || null
  }))
}

export function droppedPictures(transfer: DataTransfer | null): TransferredPicture[] {
  return pictureFilesOf(transfer).map((file) => ({ file, name: file.name || null }))
}

/**
 * How many pictures are being dragged. Before the drop a browser may show only that files are
 * coming, which counts as one picture.
 */
export function draggedPictureCount(transfer: DataTransfer | null): number {
  if (!transfer?.types.includes('Files')) return 0
  const files = Array.from(transfer.items ?? []).filter((item) => item.kind === 'file')
  if (files.length === 0) return 1
  return files.filter((item) => item.type === '' || item.type.startsWith('image/')).length
}
