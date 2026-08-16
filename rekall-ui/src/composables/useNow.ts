import { onMounted, onUnmounted, ref } from 'vue'

/**
 * A clock, ticking only while the component holding it is on screen.
 *
 * A running timer needs something to recompute its elapsed time against every second; nothing
 * else here does, so the interval lives and dies with the component rather than the store.
 */
export function useNow(intervalMs = 1000) {
  const now = ref(Date.now())
  let handle: ReturnType<typeof setInterval> | undefined

  onMounted(() => {
    handle = setInterval(() => {
      now.value = Date.now()
    }, intervalMs)
  })

  onUnmounted(() => clearInterval(handle))

  return now
}
