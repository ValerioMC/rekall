<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { fetchClaudeUsage } from '@/api/claude.api'
import { formatResetIn } from '@/common/format/countdown'
import { relativeTime } from '@/common/format/relative-time'
import { useNow } from '@/composables/useNow'
import {
  claudeSessionUsage,
  type ClaudeUsage,
  type ClaudeUsageLimit,
  type ClaudeUsageSeverity
} from '@/model/claude'

/**
 * The top-bar meter for the logged-in account's Claude usage: a ring and the session percentage.
 *
 * The ring is the whole vocabulary. Filled, it is the session window. Sweeping, a reading is on
 * its way. Dashed, there is no reading to draw, and the meter's one job becomes getting one:
 * the trigger itself checks again, and so does the button in the popover, because a token that
 * was not readable when the app opened (a keychain still locked, a login that happened after
 * launch) is the usual way this meter goes blank.
 */

/** A good reading is refreshed every minute; a blank one is retried more often than that. */
const POLL_MS = 60_000
const RETRY_MS = 15_000
const RING_RADIUS = 8
const RING_CIRCUMFERENCE = 2 * Math.PI * RING_RADIUS
/** How much of the ring the sweep draws while a reading is in flight. */
const SWEEP_ARC = RING_CIRCUMFERENCE * 0.28

const usage = ref<ClaudeUsage | null>(null)
const reachable = ref(true)
const reading = ref(false)
const lastReadAt = ref<string | null>(null)
const open = ref(false)

const now = useNow(15_000)
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

const state = computed<'ok' | 'reading' | 'signed-out' | 'offline'>(() => {
  const current = usage.value
  if (!current && reading.value) return 'reading'
  if (!reachable.value || !current) return 'offline'
  if (current.status === 'UNAUTHENTICATED') return 'signed-out'
  if (current.status === 'OK' && current.limits.length > 0) return 'ok'
  return 'offline'
})

const blank = computed(() => state.value === 'signed-out' || state.value === 'offline')

function tick(): void {
  if (document.hidden) return
  const since = lastReadAt.value
    ? Date.now() - Date.parse(lastReadAt.value)
    : Number.POSITIVE_INFINITY
  if (since >= (state.value === 'ok' ? POLL_MS : RETRY_MS)) void load()
}

/** A window brought back to the front has missed its polls; it reads rather than waits. */
function onVisibilityChange(): void {
  if (!document.hidden) tick()
}

onMounted(() => {
  void load()
  timer = setInterval(tick, RETRY_MS)
  document.addEventListener('visibilitychange', onVisibilityChange)
})

onBeforeUnmount(() => {
  clearInterval(timer)
  document.removeEventListener('visibilitychange', onVisibilityChange)
})

const limits = computed<readonly ClaudeUsageLimit[]>(() => usage.value?.limits ?? [])
const session = computed<ClaudeUsageLimit | null>(() =>
  usage.value ? claudeSessionUsage(usage.value) : null
)

const TEXT: Readonly<Record<ClaudeUsageSeverity, string>> = {
  NORMAL: 'text-anchor',
  WARNING: 'text-warn',
  CRITICAL: 'text-danger'
}
const FILL: Readonly<Record<ClaudeUsageSeverity, string>> = {
  NORMAL: 'bg-anchor',
  WARNING: 'bg-warn',
  CRITICAL: 'bg-danger'
}
const STROKE: Readonly<Record<ClaudeUsageSeverity, string>> = {
  NORMAL: 'stroke-anchor',
  WARNING: 'stroke-warn',
  CRITICAL: 'stroke-danger'
}

const severity = computed<ClaudeUsageSeverity>(() => session.value?.severity ?? 'NORMAL')

const ringOffset = computed(() => {
  const percent = session.value ? Math.min(100, Math.max(0, session.value.percent)) : 0
  return RING_CIRCUMFERENCE * (1 - percent / 100)
})

function percentLabel(limit: ClaudeUsageLimit): string {
  return `${Math.round(limit.percent)}%`
}

function resetLabel(limit: ClaudeUsageLimit): string {
  const left = formatResetIn(limit.resetsAt, now.value)
  return left ? `resets in ${left}` : 'no scheduled reset'
}

const checkedLabel = computed(() =>
  lastReadAt.value ? `Checked ${relativeTime(lastReadAt.value, now.value)}` : ''
)

const triggerLabel = computed(() => {
  if (state.value === 'reading') return 'Claude usage: taking a reading'
  if (state.value === 'signed-out') return 'Claude usage: sign in to Claude Code. Check again'
  if (state.value === 'offline') return 'Claude usage: no reading. Check again'
  const current = session.value
  if (!current) return 'Claude usage'
  const left = formatResetIn(current.resetsAt, now.value)
  return `Claude session at ${percentLabel(current)}${left ? `, resets in ${left}` : ''}`
})

/** With a figure the trigger opens the detail; without one, it goes and gets a figure. */
function onTrigger(): void {
  if (blank.value) {
    open.value = true
    void load(true)
    return
  }
  open.value = !open.value
}

function checkAgain(): void {
  void load(true)
}

function close(): void {
  open.value = false
}
</script>

<template>
  <div
    class="relative"
    data-testid="claude-usage"
    :data-state="state"
    @mouseenter="open = true"
    @mouseleave="close"
    @focusin="open = true"
    @focusout="close"
  >
    <button
      type="button"
      class="focus-ring group flex h-8 items-center gap-2 rounded-[var(--radius-control)] border border-border-strong bg-surface-raised px-2.5 text-[12px] text-text-subtle transition-colors hover:border-accent hover:bg-surface-hover"
      :aria-label="triggerLabel"
      :aria-expanded="open"
      :aria-busy="reading"
      @click="onTrigger"
      @keydown.esc="close"
    >
      <span class="relative grid size-[22px] place-items-center" aria-hidden="true">
        <svg class="size-[22px] -rotate-90" viewBox="0 0 22 22" fill="none">
          <circle
            cx="11"
            cy="11"
            :r="RING_RADIUS"
            :class="blank && !reading ? 'stroke-text-subtle' : 'stroke-border'"
            stroke-width="2.5"
            :stroke-dasharray="blank && !reading ? '1.5 3.2' : undefined"
            stroke-linecap="round"
          />
          <circle
            v-if="state === 'ok' && session && !reading"
            cx="11"
            cy="11"
            :r="RING_RADIUS"
            class="transition-[stroke-dashoffset] duration-500"
            :class="STROKE[severity]"
            stroke-width="2.5"
            stroke-linecap="round"
            :stroke-dasharray="RING_CIRCUMFERENCE"
            :stroke-dashoffset="ringOffset"
          />
          <circle
            v-else-if="reading"
            cx="11"
            cy="11"
            :r="RING_RADIUS"
            class="ring-sweep stroke-anchor"
            data-testid="usage-sweep"
            stroke-width="2.5"
            stroke-linecap="round"
            :stroke-dasharray="`${SWEEP_ARC} ${RING_CIRCUMFERENCE}`"
          />
        </svg>
      </span>

      <template v-if="state === 'ok' && session">
        <span class="font-mono tabular-nums" :class="TEXT[severity]">
          {{ percentLabel(session) }}
        </span>
        <span class="hidden font-mono text-[11px] text-text-subtle sm:inline">
          {{ formatResetIn(session.resetsAt, now) }}
        </span>
      </template>

      <template v-else-if="state === 'reading'">
        <span>Reading usage</span>
      </template>

      <template v-else>
        <span>{{ state === 'signed-out' ? 'Sign in' : 'No reading' }}</span>
        <svg
          class="size-3 text-text-subtle transition-colors group-hover:text-accent"
          :class="{ 'opacity-0': reading }"
          viewBox="0 0 12 12"
          fill="none"
          aria-hidden="true"
        >
          <path
            d="M10 6A4 4 0 1 1 8.6 2.95"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
          />
          <path
            d="M8.2 1.4 9.4 3.1 7.5 3.9"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </template>
    </button>

    <div
      v-if="open"
      class="rise absolute right-0 top-full z-(--z-overlay) mt-2 w-[264px] rounded-[var(--radius-card)] border border-border-strong bg-surface-raised p-3 shadow-modal"
      role="group"
      aria-label="Claude usage"
    >
      <div class="mb-2.5 flex items-center justify-between gap-2">
        <p class="eyebrow flex items-center gap-1.5">
          <span class="h-2.5 w-[3px] shrink-0 rounded-full bg-anchor" aria-hidden="true" />
          Claude usage
        </p>
        <button
          v-if="state === 'ok'"
          type="button"
          class="focus-ring rounded-md px-1.5 py-0.5 text-[11px] text-text-subtle transition-colors hover:bg-surface-hover hover:text-text disabled:opacity-50"
          data-testid="usage-check-again"
          :disabled="reading"
          @click="checkAgain"
        >
          {{ reading ? 'Checking…' : 'Check again' }}
        </button>
      </div>

      <ul v-if="state === 'ok'" class="flex flex-col gap-3">
        <li v-for="limit in limits" :key="limit.key" class="flex flex-col gap-1.5">
          <div class="flex items-baseline justify-between gap-2">
            <span class="text-[12px] text-text-muted">{{ limit.label }}</span>
            <span class="font-mono text-[12px] tabular-nums" :class="TEXT[limit.severity]">
              {{ percentLabel(limit) }}
            </span>
          </div>
          <div class="h-1.5 w-full overflow-hidden rounded-full bg-border">
            <span
              class="block h-full rounded-full transition-[width] duration-500"
              :class="FILL[limit.severity]"
              :style="{ width: `${Math.min(100, Math.max(2, limit.percent))}%` }"
            />
          </div>
          <span class="font-mono text-[10.5px] text-text-subtle">{{ resetLabel(limit) }}</span>
        </li>
      </ul>

      <p v-else-if="state === 'reading'" class="text-[12px] leading-relaxed text-text-subtle">
        Taking a reading from Anthropic.
      </p>

      <template v-else>
        <p class="text-[12px] leading-relaxed text-text-subtle">
          <template v-if="state === 'signed-out'">
            No Claude Code login was found. Sign in from a Claude Code terminal, then check again.
          </template>
          <template v-else>
            Anthropic could not be reached. The meter retries on its own every 15 seconds.
          </template>
        </p>
        <div class="mt-3 flex items-center justify-between gap-2">
          <span class="font-mono text-[10.5px] text-text-subtle">{{ checkedLabel }}</span>
          <button
            type="button"
            class="focus-ring inline-flex h-7 items-center gap-1.5 rounded-[var(--radius-control)] border border-anchor-line bg-anchor-soft px-2.5 text-[12px] text-anchor transition-colors hover:border-anchor hover:bg-anchor/15 disabled:opacity-50"
            data-testid="usage-check-again"
            :disabled="reading"
            @click="checkAgain"
          >
            {{ reading ? 'Checking…' : 'Check again' }}
          </button>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
/* A reading in flight: a short arc going round the ring once a second and a bit. */
.ring-sweep {
  transform-origin: 11px 11px;
  animation: ring-sweep 1.2s linear infinite;
}

@keyframes ring-sweep {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
