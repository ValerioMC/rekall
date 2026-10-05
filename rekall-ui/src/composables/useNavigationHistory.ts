import { onUnmounted, readonly, ref, type Ref } from 'vue'
import { useRouter } from 'vue-router'

export interface NavigationHistory {
  readonly canGoBack: Readonly<Ref<boolean>>
  readonly canGoForward: Readonly<Ref<boolean>>
  goBack(): void
  goForward(): void
}

/**
 * The reader's back and forward reach, read from the router's own history entry, which records
 * the entry on each side of it. The arrows therefore follow the real history stack, reloads and
 * the mouse's back button included, with no second record kept beside it.
 */
export function useNavigationHistory(): NavigationHistory {
  const router = useRouter()
  const canGoBack = ref(false)
  const canGoForward = ref(false)

  function readReach(): void {
    const entry = router.options.history.state
    canGoBack.value = typeof entry.back === 'string'
    canGoForward.value = typeof entry.forward === 'string'
  }

  readReach()
  onUnmounted(router.afterEach(readReach))

  return {
    canGoBack: readonly(canGoBack),
    canGoForward: readonly(canGoForward),
    goBack: () => {
      if (canGoBack.value) router.back()
    },
    goForward: () => {
      if (canGoForward.value) router.forward()
    }
  }
}
