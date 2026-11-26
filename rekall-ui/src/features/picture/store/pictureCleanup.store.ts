import { defineStore } from 'pinia'
import { ref } from 'vue'
import { isConsoleInBackground, postNotice } from '@/common/native/notify'
import {
  discardPictureList,
  fetchPictureCleanup,
  startPictureDeletion,
  startPictureSearch
} from '@/features/picture/api/picture-cleanup.api'
import { usePictureCleanupStream } from '@/features/picture/composables/usePictureCleanupStream'
import { endingOf, type CleanupEnding } from '@/features/picture/lib/picture-cleanup-wording'
import type { PictureCleanupStatus, PictureCleanupStreamEvent } from '@/features/picture/model/picture-cleanup'
import { useToastStore } from '@/stores/toast.store'

/**
 * The cleanup of unused pictures as the console last heard of it; null until first read. It follows
 * the event stream from the moment it exists and tells each ending once, on whatever screen is open.
 */
export const usePictureCleanupStore = defineStore('pictureCleanup', () => {
  const status = ref<PictureCleanupStatus | null>(null)
  const toast = useToastStore()
  let revision = 0

  /** An answer overtaken by a later request or frame is returned but not shown. */
  async function load(): Promise<PictureCleanupStatus> {
    const asked = ++revision
    const answered = await fetchPictureCleanup()
    if (asked === revision) status.value = answered
    return answered
  }

  /** A listed result, even an empty one, has to be discarded before another search may start. */
  async function search(): Promise<void> {
    if (status.value?.state === 'READY') await discardPictureList()
    await startPictureSearch()
    await load()
  }

  async function deleteListed(): Promise<void> {
    await startPictureDeletion()
    await load()
  }

  async function discard(): Promise<void> {
    await discardPictureList()
    await load()
  }

  /** Progress travels in the frame; every other state has more to show, which is read again. */
  function follow(frame: PictureCleanupStreamEvent): void {
    if (frame.state === 'SEARCHING' || frame.state === 'DELETING') {
      revision += 1
      status.value = { state: frame.state, percentage: frame.percentage }
      return
    }
    const read = frame.state === 'IDLE' ? load() : reportEnding()
    read.catch((caught: unknown) => toast.notifyError(caught))
  }

  async function reportEnding(): Promise<void> {
    const ending = endingOf(await load())
    if (ending) announce(ending)
  }

  function announce({ title, detail, failed }: CleanupEnding): void {
    if (failed) toast.notifyError(`${title}: ${detail}`)
    else toast.notify(detail)
    if (isConsoleInBackground()) void postNotice({ title, body: detail })
  }

  usePictureCleanupStream(follow)

  return { status, load, search, deleteListed, discard }
})
