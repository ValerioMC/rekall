import { onBeforeUnmount, ref, watch } from 'vue'
import type { Ref } from 'vue'
import { searchText } from '@/features/search/api/search.api'
import { isTextQuery } from '@/features/search/model/search'
import type { SearchHit } from '@/features/search/model/search'

const SEARCH_DEBOUNCE_MS = 200

/**
 * Searches the text behind the records (descriptions, steps, wrapups and notes) once typing
 * pauses. A query that is only anchors is never searched: the titles already resolve it.
 */
export function useTextSearch(query: Ref<string>) {
  const hits = ref<SearchHit[]>([])
  const searching = ref(false)
  const failed = ref(false)
  let timer: ReturnType<typeof setTimeout> | null = null
  let sequence = 0

  async function run(term: string, mine: number): Promise<void> {
    try {
      const found = await searchText(term)
      if (mine !== sequence) return
      hits.value = found
      failed.value = false
    } catch {
      if (mine !== sequence) return
      hits.value = []
      failed.value = true
    } finally {
      if (mine === sequence) searching.value = false
    }
  }

  watch(
    query,
    (value) => {
      if (timer) clearTimeout(timer)
      const mine = ++sequence
      if (!isTextQuery(value)) {
        hits.value = []
        searching.value = false
        failed.value = false
        return
      }
      searching.value = true
      timer = setTimeout(() => void run(value.trim(), mine), SEARCH_DEBOUNCE_MS)
    },
    { flush: 'sync' }
  )

  onBeforeUnmount(() => {
    if (timer) clearTimeout(timer)
    sequence += 1
  })

  return { hits, searching, failed }
}
