import { onBeforeUnmount, onMounted } from 'vue'
import { isTypingTarget } from '@/features/steps/lib/step-navigation'
import { reviewKeyAction, type ReviewKeyAction } from '@/features/review/lib/review-keys'
import { useSendBackGuardStore } from '@/features/review/store/sendBackGuard.store'

/** Binds the Review keys while the screen is mounted. They are ignored while a field has focus or the send-back question is open. */
export function useReviewKeys(handlers: Readonly<Record<ReviewKeyAction, () => void>>): void {
  const sendBackGuard = useSendBackGuardStore()

  function onKeydown(event: KeyboardEvent): void {
    if (isTypingTarget(event.target) || sendBackGuard.pending) return
    const action = reviewKeyAction(event)
    if (action === null) return
    event.preventDefault()
    handlers[action]()
  }

  onMounted(() => window.addEventListener('keydown', onKeydown))
  onBeforeUnmount(() => window.removeEventListener('keydown', onKeydown))
}
