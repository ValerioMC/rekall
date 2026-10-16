<script setup lang="ts">
import AppButton from '@/components/ui/AppButton.vue'
import { computed, ref } from 'vue'
import { useClaudeUsageReading } from '@/features/claude/composables/useClaudeUsageReading'
import { formatResetIn } from '@/common/format/countdown'
import { relativeTime } from '@/common/format/relative-time'
import { useNow } from '@/composables/useNow'
import {
  claudeSessionUsage,
  type ClaudeUsageLimit,
  type ClaudeUsageSeverity
} from '@/features/claude/model/claude'

/**
 * The top-bar meter for the logged-in account's Claude usage: a ring and the session percentage.
 *
 * The ring is the whole vocabulary. Filled, it is the session window. Sweeping, a reading is on
 * its way. Dashed, there is no reading to draw, and the meter's one job becomes getting one:
 * the trigger itself checks again, and so does the button in the popover, because a token that
 * was not readable when the app opened (a keychain still locked, a login that happened after
 * launch) is the usual way this meter goes blank. When readings happen is `useClaudeUsageReading`.
 */

const RING_RADIUS = 8
const RING_CIRCUMFERENCE = 2 * Math.PI * RING_RADIUS
/** How much of the ring the sweep draws while a reading is in flight. */
const SWEEP_ARC = RING_CIRCUMFERENCE * 0.28

const open = ref(false)

const now = useNow(15_000)
const { usage, lastReadAt, reading, state, blank, waiting, waitLabel, load } = useClaudeUsageReading(now)

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

const figuresAge = computed(() => (usage.value ? relativeTime(usage.value.fetchedAt, now.value) : ''))

const checkedLabel = computed(() =>
  lastReadAt.value ? `Checked ${relativeTime(lastReadAt.value, now.value)}` : ''
)

const triggerLabel = computed(() => {
  if (state.value === 'reading') return 'Claude usage: taking a reading'
  if (state.value === 'signed-out') return 'Claude usage: sign in to Claude Code. Check again'
  if (state.value === 'rate-limited')
    return `Claude usage: Anthropic asked to wait, next reading in ${waitLabel.value}`
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
    if (!waiting.value) void load(true)
    return
  }
  open.value = !open.value
}

function checkAgain(): void {
  if (!waiting.value) void load(true)
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
      class="status-segment focus-ring group text-text-muted hover:text-text"
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
            class="transition-[stroke-dashoffset] duration-(--motion-slow)"
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
        <span class="hidden font-mono text-caption text-text-subtle min-[1600px]:inline">
          {{ formatResetIn(session.resetsAt, now) }}
        </span>
      </template>

      <template v-else-if="state === 'reading'">
        <span>Reading usage</span>
      </template>

      <template v-else>
        <span v-if="state === 'signed-out'">Sign in</span>
        <span v-else-if="state === 'rate-limited'" class="font-mono tabular-nums">
          Wait {{ waitLabel }}
        </span>
        <span v-else>No reading</span>
        <svg
          v-if="state !== 'rate-limited'"
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
      class="rise absolute right-0 top-full z-(--z-overlay) mt-2 w-[264px] rounded-card border border-border-strong bg-surface-raised p-3 shadow-modal"
      role="group"
      aria-label="Claude usage"
    >
      <div class="mb-2.5 flex items-center justify-between gap-2">
        <p class="eyebrow flex items-center gap-1.5">
          <span class="h-2.5 w-[3px] shrink-0 rounded-full bg-anchor" aria-hidden="true" />
          Claude usage
        </p>
        <AppButton
          v-if="state === 'ok'"
          variant="quiet"
          size="sm"
          data-testid="usage-check-again"
          :disabled="reading || waiting"
          @click="checkAgain"
        >
          {{ reading ? 'Checking…' : 'Check again' }}
        </AppButton>
      </div>

      <p
        v-if="state === 'ok' && waiting"
        class="mb-3 rounded-control border border-warn/40 bg-warn/10 px-2 py-1.5 text-caption leading-relaxed text-text-muted"
        data-testid="usage-hold"
      >
        Figures from {{ figuresAge }}. Anthropic asked this machine to wait; the next reading can
        be taken in {{ waitLabel }}.
      </p>

      <ul v-if="state === 'ok'" class="flex flex-col gap-3">
        <li v-for="limit in limits" :key="limit.key" class="flex flex-col gap-1.5">
          <div class="flex items-baseline justify-between gap-2">
            <span class="text-small text-text-muted">{{ limit.label }}</span>
            <span class="font-mono text-small tabular-nums" :class="TEXT[limit.severity]">
              {{ percentLabel(limit) }}
            </span>
          </div>
          <div class="h-1.5 w-full overflow-hidden rounded-full bg-border">
            <span
              class="block h-full rounded-full transition-[width] duration-(--motion-slow)"
              :class="FILL[limit.severity]"
              :style="{ width: `${Math.min(100, Math.max(2, limit.percent))}%` }"
            />
          </div>
          <span class="font-mono text-micro text-text-subtle">{{ resetLabel(limit) }}</span>
        </li>
      </ul>

      <p v-else-if="state === 'reading'" class="text-small leading-relaxed text-text-subtle">
        Taking a reading from Anthropic.
      </p>

      <template v-else>
        <p class="text-small leading-relaxed text-text-subtle">
          <template v-if="state === 'signed-out'">
            No valid Claude Code login was found. Open a Claude Code terminal, which signs in or
            refreshes the token, then check again.
          </template>
          <template v-else-if="state === 'rate-limited'">
            Anthropic asked this machine to wait before reading usage again. The meter will take
            the next reading in {{ waitLabel }}; asking sooner only extends the wait.
          </template>
          <template v-else>
            Anthropic could not be reached. The meter checks again every five minutes while the
            console is on screen, and when the window comes back to the front.
          </template>
        </p>
        <div class="mt-3 flex items-center justify-between gap-2">
          <span class="font-mono text-micro text-text-subtle">{{ checkedLabel }}</span>
          <button
            v-if="state !== 'rate-limited'"
            type="button"
            class="key-quiet focus-ring inline-flex h-7 items-center gap-1.5 rounded-key px-2.5 text-small disabled:opacity-50"
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
