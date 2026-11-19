import { defineStore } from 'pinia'
import { ref } from 'vue'
import { usePreferencesStore } from '@/features/settings/store/preferences.store'

interface PendingSendBack {
  readonly subject: string
  readonly run: () => void | Promise<void>
}

/**
 * Sending work back asks first when Settings says so. Every send back goes through `guard`, and
 * the one dialog mounted by Bootstrap shows what is pending.
 */
export const useSendBackGuardStore = defineStore('sendBackGuard', () => {
  const preferences = usePreferencesStore()
  const pending = ref<PendingSendBack | null>(null)

  async function guard(subject: string, run: () => void | Promise<void>): Promise<void> {
    if (!preferences.reviewConfirmSendBack) {
      await run()
      return
    }
    pending.value = { subject, run }
  }

  async function confirm(): Promise<void> {
    const waiting = pending.value
    pending.value = null
    await waiting?.run()
  }

  function cancel(): void {
    pending.value = null
  }

  return { pending, guard, confirm, cancel }
})
