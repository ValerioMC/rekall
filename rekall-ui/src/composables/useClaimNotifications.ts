import { watch } from 'vue'
import type { Ref } from 'vue'
import { postNotice } from '@/common/native/notify'
import type { ReviewItem } from '@/model/review'

/**
 * Posts a system notification when something joins the review queue while nobody is looking at
 * the console: the window is hidden or another app has the focus. What was already waiting when
 * the console loaded is the baseline and never notifies; only arrivals after it do.
 */
export function useClaimNotifications(queue: Readonly<Ref<readonly ReviewItem[]>>, isLoading: Readonly<Ref<boolean>>) {
  let known: Set<string> | null = null

  watch(
    [queue, isLoading],
    ([items, loading]) => {
      const keys = new Set(items.map((item) => item.key))
      if (known === null) {
        if (!loading) known = keys
        return
      }
      const arrived = items.filter((item) => !known!.has(item.key))
      known = keys
      if (arrived.length === 0) return
      if (document.visibilityState === 'visible' && document.hasFocus()) return
      void postNotice(noticeFor(arrived))
    },
    { immediate: true }
  )
}

export function noticeFor(arrived: readonly ReviewItem[]): { title: string; body: string } {
  const first = arrived[0]!
  if (arrived.length > 1) {
    return {
      title: `${arrived.length} pieces of work are waiting for review`,
      body: arrived.map((item) => item.stepTitle ?? item.taskTitle).join(' · ')
    }
  }
  return first.stepTitle
    ? { title: 'A step is waiting for review', body: `${first.stepTitle} — ${first.taskTitle}` }
    : { title: 'A task is waiting for review', body: first.taskTitle }
}
