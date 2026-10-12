import { ref, type Ref } from 'vue'

/**
 * Stands between a person and the edits they have not saved. `guard` runs an action at once when
 * nothing is unsaved, and otherwise holds it until `confirm`; `askToLeave` is the same question
 * for a route change, answered by a promise the router waits on.
 */
export function useDiscardGuard(isDirty: Ref<boolean>) {
  const asking = ref(false)
  let held: (() => void) | null = null
  let leaving: ((leave: boolean) => void) | null = null

  function guard(action: () => void): void {
    if (!isDirty.value) {
      action()
      return
    }
    held = action
    asking.value = true
  }

  function askToLeave(): Promise<boolean> {
    if (!isDirty.value) return Promise.resolve(true)
    asking.value = true
    return new Promise((resolve) => {
      leaving = resolve
    })
  }

  function confirm(): void {
    asking.value = false
    const action = held
    const resolve = leaving
    held = null
    leaving = null
    action?.()
    resolve?.(true)
  }

  function cancel(): void {
    asking.value = false
    held = null
    leaving?.(false)
    leaving = null
  }

  return { asking, guard, askToLeave, confirm, cancel }
}
