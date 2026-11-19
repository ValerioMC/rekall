import { computed, type WritableComputedRef } from 'vue'
import { useRoute, useRouter } from 'vue-router'

export type WorkLayout = 'ledger' | 'board'

/** The Work layout is the route: `/` is the ledger and `/work/board` the board, so the arrows and a link both reach either. */
export function useWorkLayout(): WritableComputedRef<WorkLayout> {
  const route = useRoute()
  const router = useRouter()
  return computed<WorkLayout>({
    get: () => (route.name === 'work-board' ? 'board' : 'ledger'),
    set: (layout) => void router.push(layout === 'board' ? '/work/board' : '/')
  })
}
