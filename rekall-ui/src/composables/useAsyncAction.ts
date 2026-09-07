import { ref } from 'vue'
import { useToastStore } from '@/stores/toast.store'

export function useAsyncAction() {
  const toast = useToastStore()
  const isRunning = ref(false)

  async function run<T>(operation: () => Promise<T>, successMessage?: string): Promise<T | null> {
    isRunning.value = true
    try {
      const result = await operation()
      if (successMessage) toast.notify(successMessage)
      return result
    } catch (error) {
      toast.notifyError(error)
      return null
    } finally {
      isRunning.value = false
    }
  }

  return { isRunning, run }
}
