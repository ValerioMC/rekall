import { onScopeDispose, ref, watch, type Ref } from 'vue'
import type { Decision } from '@/features/decisions/model/decision'
import type { DecisionId, TaskId } from '@/model/branded'

const FRESH_FOR_MS = 2800

/**
 * Which decisions arrived or changed since the list was last read, for a moment, so a person
 * working in the terminal sees where the record moved. The first read of a task marks nothing.
 */
export function useFreshDecisions(taskId: Ref<TaskId | null>, decisions: Ref<readonly Decision[]>): {
  fresh: Ref<ReadonlySet<DecisionId>>
  announcement: Ref<string>
} {
  const fresh = ref<ReadonlySet<DecisionId>>(new Set())
  const announcement = ref('')
  let known: Map<DecisionId, string> | null = null
  let clearing: ReturnType<typeof setTimeout> | undefined

  function signatures(list: readonly Decision[]): Map<DecisionId, string> {
    return new Map(list.map((decision) => [decision.id, JSON.stringify(decision)]))
  }

  function mark(changed: readonly Decision[]): void {
    fresh.value = new Set([...fresh.value, ...changed.map((decision) => decision.id)])
    announcement.value = changed.length === 1 ? `Decision ${changed[0]!.key} updated` : `${changed.length} decisions updated`
    clearTimeout(clearing)
    clearing = setTimeout(() => {
      fresh.value = new Set()
    }, FRESH_FOR_MS)
  }

  watch(taskId, () => {
    known = null
    fresh.value = new Set()
    announcement.value = ''
  })

  watch(
    decisions,
    (list) => {
      const next = signatures(list)
      if (known !== null) {
        const before = known
        const changed = list.filter((decision) => before.get(decision.id) !== next.get(decision.id))
        if (changed.length > 0) mark(changed)
      }
      known = next
    },
    { immediate: true }
  )

  onScopeDispose(() => clearTimeout(clearing))

  return { fresh, announcement }
}
