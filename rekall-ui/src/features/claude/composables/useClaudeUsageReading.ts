import { computed, onBeforeUnmount, onMounted, ref, type ComputedRef, type Ref } from 'vue'
import { fetchClaudeUsage } from '@/features/claude/api/claude.api'
import { formatResetIn } from '@/common/format/countdown'
import type { ClaudeUsage } from '@/features/claude/model/claude'

/** How old a reading may get on screen, and how old it may be when the window comes back. */
const POLL_MS = 5 * 60_000
const STALE_ON_RETURN_MS = 60_000
/** How often the meter looks at the clock; not how often it reads. */
const TICK_MS = 30_000

export type UsageState = 'ok' | 'reading' | 'signed-out' | 'rate-limited' | 'offline'

export interface ClaudeUsageReading {
  readonly usage: Ref<ClaudeUsage | null>
  readonly lastReadAt: Ref<string | null>
  /** A reading is in flight. */
  readonly reading: Ref<boolean>
  readonly state: ComputedRef<UsageState>
  /** There is no figure to draw: the meter's job is getting one. */
  readonly blank: ComputedRef<boolean>
  /** Anthropic asked this machine to wait and the wait has not run out. */
  readonly waiting: ComputedRef<boolean>
  readonly waitLabel: ComputedRef<string>
  load(refresh?: boolean): Promise<void>
}

/**
 * When the account's Claude usage is read: once when the console opens, every few minutes while
 * it is on screen, when the window returns after a minute away, and whenever asked. Nothing
 * retries faster, because Anthropic rate limits a client that keeps asking; the server says
 * until when, and the reading waits it out.
 */
export function useClaudeUsageReading(now: Ref<number>): ClaudeUsageReading {
  const usage = ref<ClaudeUsage | null>(null)
  const reachable = ref(true)
  const reading = ref(false)
  const lastReadAt = ref<string | null>(null)
  let timer: ReturnType<typeof setInterval> | undefined

  async function load(refresh = false): Promise<void> {
    if (reading.value) return
    reading.value = true
    try {
      usage.value = await fetchClaudeUsage(refresh)
      reachable.value = true
    } catch {
      reachable.value = false
    } finally {
      reading.value = false
      lastReadAt.value = new Date().toISOString()
    }
  }

  const state = computed<UsageState>(() => {
    const current = usage.value
    if (!current && reading.value) return 'reading'
    if (!reachable.value || !current) return 'offline'
    if (current.status === 'UNAUTHENTICATED') return 'signed-out'
    if (current.status === 'RATE_LIMITED') return 'rate-limited'
    if (current.status === 'OK' && current.limits.length > 0) return 'ok'
    return 'offline'
  })

  const blank = computed(
    () => state.value === 'signed-out' || state.value === 'rate-limited' || state.value === 'offline'
  )

  const retryAt = computed<string | null>(() => usage.value?.retryAt ?? null)
  const waiting = computed(() => {
    const until = retryAt.value
    return until !== null && Date.parse(until) > now.value
  })
  const waitLabel = computed(() => (retryAt.value ? formatResetIn(retryAt.value, now.value) : ''))

  function readIfOlderThan(maxAge: number): void {
    if (document.hidden || waiting.value) return
    const age = lastReadAt.value ? Date.now() - Date.parse(lastReadAt.value) : Number.POSITIVE_INFINITY
    if (age >= maxAge) void load()
  }

  /** A hold that has just lapsed on a blank meter is read at once; everything else waits its turn. */
  function tick(): void {
    readIfOlderThan(state.value === 'rate-limited' ? 0 : POLL_MS)
  }

  function onVisibilityChange(): void {
    if (!document.hidden) readIfOlderThan(STALE_ON_RETURN_MS)
  }

  onMounted(() => {
    void load()
    timer = setInterval(tick, TICK_MS)
    document.addEventListener('visibilitychange', onVisibilityChange)
    window.addEventListener('focus', onVisibilityChange)
  })

  onBeforeUnmount(() => {
    clearInterval(timer)
    document.removeEventListener('visibilitychange', onVisibilityChange)
    window.removeEventListener('focus', onVisibilityChange)
  })

  return { usage, lastReadAt, reading, state, blank, waiting, waitLabel, load }
}
