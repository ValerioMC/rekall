import { onMounted, onUnmounted, ref } from 'vue'

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
