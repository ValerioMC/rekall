import { onBeforeUnmount, ref, type Ref } from 'vue'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import type { TargetSuggestion } from '@/features/connection/model/connection'
import type { ConnectionId } from '@/model/branded'

const TYPING_PAUSE_MS = 250

/**
 * The Jira projects or Confluence spaces a target can be filled with, asked of the site while a
 * key is typed. A newer question replaces an older answer that arrives late; one the site could not
 * answer leaves its reason to show beside the field instead of a toast on every key.
 */
export function useTargetSuggestions(connectionId: () => ConnectionId, isAsked: () => boolean) {
  const store = useConnectionsStore()
  const suggestions: Ref<readonly TargetSuggestion[]> = ref([])
  const unavailable = ref<string | null>(null)
  let pause: ReturnType<typeof setTimeout> | null = null
  let latest = 0

  function ask(query: string): void {
    if (!isAsked()) return
    if (pause) clearTimeout(pause)
    pause = setTimeout(() => void load(query.trim()), TYPING_PAUSE_MS)
  }

  async function load(query: string): Promise<void> {
    const question = ++latest
    try {
      const found = await store.targetSuggestions(connectionId(), query)
      if (question !== latest) return
      suggestions.value = found
      unavailable.value = null
    } catch (failure) {
      if (question !== latest) return
      suggestions.value = []
      unavailable.value = failure instanceof Error ? failure.message : 'The site could not be asked.'
    }
  }

  onBeforeUnmount(() => {
    if (pause) clearTimeout(pause)
  })

  return { suggestions, unavailable, ask }
}
