import { onMounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useSchemaStore } from '@/stores/schema.store'
import { usePlanStore } from '@/stores/plan.store'
import { useToastStore } from '@/stores/toast.store'

/**
 * The schema plus the pending-change count, loaded once and shared.
 *
 * The two travel together because every schema edit changes both, and a badge that disagrees
 * with the screen next to it is worse than no badge.
 */
export function useSchema() {
  const schema = useSchemaStore()
  const plan = usePlanStore()
  const toast = useToastStore()

  const { entities, relations, appliedEntities, isLoading, error } = storeToRefs(schema)
  const { pendingCount } = storeToRefs(plan)

  async function reload(): Promise<void> {
    try {
      await Promise.all([schema.load(), plan.refresh()])
    } catch (e) {
      toast.notifyError(e)
    }
  }

  onMounted(reload)

  return { entities, relations, appliedEntities, isLoading, error, pendingCount, reload }
}
