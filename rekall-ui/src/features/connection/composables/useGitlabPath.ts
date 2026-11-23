import { computed, onBeforeUnmount, ref, type Ref } from 'vue'
import { cursorOf, pathOf } from '@/features/connection/lib/gitlab-path'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import type { GitlabNamespace, TargetCheck } from '@/features/connection/model/connection'
import type { ConnectionId } from '@/model/branded'

const TYPING_PAUSE_MS = 250

export type PathVerdict =
  | { readonly state: 'blank' }
  | { readonly state: 'checking'; readonly path: string }
  | { readonly state: 'answered'; readonly path: string; readonly check: TargetCheck }

/**
 * A GitLab path typed or chosen for a binding. Typing lists what sits under the group named so far
 * and asks the instance about the whole path; choosing a group goes into it, choosing a project
 * completes the path. A newer question replaces an older answer that arrives late, and a listing the
 * instance could not give leaves its reason to show in the list instead of a toast on every key.
 */
export function useGitlabPath(connectionId: () => ConnectionId) {
  const store = useConnectionsStore()
  const value = ref('')
  const isOpen = ref(false)
  const rows: Ref<readonly GitlabNamespace[]> = ref([])
  const activeIndex = ref(0)
  const listingUnavailable = ref<string | null>(null)
  const verdict: Ref<PathVerdict> = ref({ state: 'blank' })
  let pause: ReturnType<typeof setTimeout> | null = null
  let latestListing = 0
  let latestCheck = 0

  const cursor = computed(() => cursorOf(value.value))
  const verified = computed(() => (verdict.value.state === 'answered' && verdict.value.check.status === 'FOUND' ? verdict.value.check.found : null))
  const isBindable = computed(() => verdict.value.state === 'blank' || verified.value !== null)

  function type(next: string): void {
    value.value = next
    isOpen.value = true
    activeIndex.value = 0
    verdict.value = pathOf(next) ? { state: 'checking', path: pathOf(next) } : { state: 'blank' }
    if (pause) clearTimeout(pause)
    pause = setTimeout(() => void ask(), TYPING_PAUSE_MS)
  }

  function seed(initial: string): void {
    value.value = initial
    verdict.value = pathOf(initial) ? { state: 'checking', path: pathOf(initial) } : { state: 'blank' }
    void checkPath(++latestCheck)
  }

  function open(): void {
    isOpen.value = true
    void listChildren(++latestListing)
  }

  function close(): void {
    isOpen.value = false
  }

  function choose(row: GitlabNamespace): void {
    value.value = row.kind === 'GROUP' ? `${row.path}/` : row.path
    activeIndex.value = 0
    if (row.kind === 'PROJECT') isOpen.value = false
    verdict.value = { state: 'checking', path: row.path }
    if (pause) clearTimeout(pause)
    void ask()
  }

  function chooseActive(): boolean {
    const row = rows.value[activeIndex.value]
    if (!row) return false
    choose(row)
    return true
  }

  function jumpTo(groupPath: string): void {
    type(`${groupPath}/`)
  }

  function move(step: number): void {
    if (!rows.value.length) return
    activeIndex.value = (activeIndex.value + step + rows.value.length) % rows.value.length
  }

  function retry(): void {
    type(value.value)
  }

  async function ask(): Promise<void> {
    await Promise.all([listChildren(++latestListing), checkPath(++latestCheck)])
  }

  async function checkPath(question: number): Promise<void> {
    const path = pathOf(value.value)
    if (!path) return
    try {
      const check = await store.targetCheck(connectionId(), path)
      if (question === latestCheck) verdict.value = { state: 'answered', path, check }
    } catch (failure) {
      const reason = failure instanceof Error ? failure.message : 'GitLab could not be asked.'
      if (question === latestCheck) verdict.value = { state: 'answered', path, check: { status: 'UNREACHABLE', reason } }
    }
  }

  async function listChildren(question: number): Promise<void> {
    const { parent, fragment } = cursor.value
    try {
      const listed = await store.gitlabChildren(connectionId(), parent, fragment)
      if (question !== latestListing) return
      rows.value = listed
      listingUnavailable.value = null
    } catch (failure) {
      if (question !== latestListing) return
      rows.value = []
      listingUnavailable.value = failure instanceof Error ? failure.message : 'GitLab could not be asked.'
    }
  }

  onBeforeUnmount(() => {
    if (pause) clearTimeout(pause)
  })

  return { value, isOpen, rows, activeIndex, cursor, listingUnavailable, verdict, verified, isBindable, type, seed, open, close, choose, chooseActive, jumpTo, move, retry }
}
