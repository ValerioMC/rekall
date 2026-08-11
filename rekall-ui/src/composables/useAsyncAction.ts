import { ref } from 'vue'
import { useToastStore } from '@/stores/toast.store'

/**
 * Runs a write, reports the outcome and tracks whether it is in flight.
 *
 * Every mutating action in this application has the same three needs: a busy flag so the
 * button can disable itself, a confirmation on success, and the server's message on failure.
 * Repeating that try/catch in every component is how one of them ends up silently swallowing
 * an error.
 */
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
