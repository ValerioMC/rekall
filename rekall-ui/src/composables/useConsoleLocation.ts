import { watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useConsoleStore } from '@/stores/console.store'
import {
  locationFromQuery,
  locationToQuery,
  sameLocation,
  type ConsoleLocation
} from '@/model/console-location'

/**
 * Keeps the console's location in its URL, one history entry per move, so the header arrows and
 * the mouse's back button walk tasks and panes as they walk screens. A bare `/` adopts what the
 * store already shows: screens that open a task select it first and then route to the console.
 */
export function useConsoleLocation(): void {
  const store = useConsoleStore()
  const route = useRoute()
  const router = useRouter()
  // The location the current entry holds, or is about to once a navigation lands.
  let inUrl: ConsoleLocation | null = null

  function writeToUrl(location: ConsoleLocation, mode: 'push' | 'replace'): void {
    inUrl = location
    const target = { name: 'console', query: locationToQuery(location) }
    void (mode === 'push' ? router.push(target) : router.replace(target))
  }

  function followRoute(): void {
    if (route.name !== 'console') return
    const wanted = locationFromQuery(route.query)
    if (wanted === null) {
      writeToUrl(store.location, 'replace')
      return
    }
    if (inUrl !== null && sameLocation(wanted, inUrl)) return
    const shown = store.showLocation(wanted)
    inUrl = shown
    // A task deleted since the entry was written is dropped; the entry says so instead of lying.
    if (!sameLocation(shown, wanted)) writeToUrl(shown, 'replace')
  }

  function followStore(location: ConsoleLocation): void {
    if (inUrl !== null && sameLocation(location, inUrl)) return
    writeToUrl(location, 'push')
  }

  // Before the catalog arrives no id in the URL can be checked, so nothing is read or written.
  watch(
    () => store.isLoading,
    (loading) => {
      if (!loading) followRoute()
    },
    { immediate: true }
  )
  watch(
    () => route.query,
    () => {
      if (!store.isLoading) followRoute()
    }
  )
  watch(
    () => store.location,
    (location) => {
      if (!store.isLoading) followStore(location)
    }
  )
}
