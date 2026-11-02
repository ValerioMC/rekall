import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { filterFromQuery, queryOf, returnToFromQuery } from '@/features/connection/lib/connection-route'
import { NO_FILTER } from '@/features/connection/lib/connection-filter'
import type { ConnectionFilter } from '@/features/connection/lib/connection-filter'
import { asConnectionId, type ConnectionId } from '@/model/branded'

/**
 * The hub's state, kept in the URL so a reload, a link and the back button all land on the same
 * view: the selected connection in the path, the filters and the way back in the query.
 */
export function useConnectionsRoute() {
  const route = useRoute()
  const router = useRouter()

  const selectedId = computed<ConnectionId | null>(() => {
    const id = route.params.id
    return typeof id === 'string' && id !== '' ? asConnectionId(id) : null
  })
  const filter = computed(() => filterFromQuery(route.query))
  const returnTo = computed(() => returnToFromQuery(route.query))

  async function select(id: ConnectionId | null): Promise<void> {
    await router.push({ name: 'connections', params: { id: id ?? undefined }, query: route.query })
  }

  async function setFilter(next: ConnectionFilter): Promise<void> {
    await router.replace({ name: 'connections', params: route.params, query: queryOf(next, returnTo.value) })
  }

  async function clearFilters(): Promise<void> {
    await setFilter(NO_FILTER)
  }

  return { selectedId, filter, returnTo, select, setFilter, clearFilters }
}
