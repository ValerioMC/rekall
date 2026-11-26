import { formatFileSize } from '@/common/format/file-size'
import { plural } from '@/features/connection/lib/connection-impact'
import type { PictureCleanupStatus } from '@/features/picture/model/picture-cleanup'

/** What a finished or failed run says wherever the reader is. */
export interface CleanupEnding {
  readonly title: string
  readonly detail: string
  readonly failed: boolean
}

/** What the panel says while no run is under way and no picture waits to be deleted, with the key that searches. */
export interface CleanupRest {
  readonly headline: string
  readonly detail: string | null
  readonly action: string
  readonly failed: boolean
}

type Finished = Extract<PictureCleanupStatus, { state: 'FINISHED' }>

/** Null for a status that ends no run. */
export function endingOf(status: PictureCleanupStatus): CleanupEnding | null {
  switch (status.state) {
    case 'READY':
      return { title: 'Search for unused images ended', detail: foundOf(status.pictures.length), failed: false }
    case 'FINISHED':
      return { title: 'Unused images deleted', detail: removedOf(status), failed: false }
    case 'FAILED':
      return { title: 'Image cleanup failed', detail: status.message, failed: true }
    default:
      return null
  }
}

/** Null while a run is under way or a list of pictures is shown. */
export function restOf(status: PictureCleanupStatus): CleanupRest | null {
  switch (status.state) {
    case 'IDLE':
      return {
        headline: 'Find the images no text shows',
        detail: 'An image stays on disk after the text showing it is deleted. Images written in the last 24 hours are left alone.',
        action: 'Find unused images',
        failed: false
      }
    case 'READY':
      if (status.pictures.length > 0) return null
      return {
        headline: foundOf(0),
        detail: 'Every image is shown by a text or was written in the last 24 hours.',
        action: 'Search again',
        failed: false
      }
    case 'FINISHED':
      return { headline: removedOf(status), detail: leftBehindOf(status), action: 'Find unused images', failed: false }
    case 'FAILED':
      return { headline: 'The cleanup failed and deleted nothing', detail: status.message, action: 'Search again', failed: true }
    default:
      return null
  }
}

export function pictureName(address: string): string {
  return address.slice(address.lastIndexOf('/') + 1)
}

function foundOf(count: number): string {
  return count === 0 ? 'No unused images found' : `${plural(count, 'unused image')} found`
}

function removedOf(status: Finished): string {
  return `${status.removed} removed, ${formatFileSize(status.freedBytes)} freed`
}

function leftBehindOf({ kept, notDeleted }: Finished): string | null {
  const sentences: string[] = []
  if (kept > 0) sentences.push(`${kept} kept: shown by a text again, or written again since the search.`)
  if (notDeleted > 0) sentences.push(`${notDeleted} could not be deleted; the server log says why.`)
  return sentences.length > 0 ? sentences.join(' ') : null
}
