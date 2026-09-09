<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { fetchClaudeUsage } from '@/api/claude.api'
import { formatResetIn } from '@/common/format/countdown'
import { useNow } from '@/composables/useNow'
import {
  claudeSessionUsage,
  type ClaudeUsage,
  type ClaudeUsageLimit,
  type ClaudeUsageSeverity
} from '@/model/claude'

const POLL_MS = 60_000
const RING_RADIUS = 8
const RING_CIRCUMFERENCE = 2 * Math.PI * RING_RADIUS

const usage = ref<ClaudeUsage | null>(null)
const reachable = ref(true)
const open = ref(false)

const now = useNow(30_000)
let timer: ReturnType<typeof setInterval> | undefined

async function load(): Promise<void> {
  try {
    usage.value = await fetchClaudeUsage()
    reachable.value = true
  } catch {
    reachable.value = false
  }
}

function tick(): void {
  if (!document.hidden) void load()
}

onMounted(() => {
  void load()
  timer = setInterval(tick, POLL_MS)
})

onBeforeUnmount(() => clearInterval(timer))

const state = computed<'ok' | 'signed-out' | 'offline'>(() => {
  if (!reachable.value) return 'offline'
  const current = usage.value
  if (!current) return 'offline'
  if (current.status === 'UNAUTHENTICATED') return 'signed-out'
  if (current.status === 'OK' && current.limits.length > 0) return 'ok'
  return 'offline'
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

const triggerLabel = computed(() => {
  if (state.value === 'signed-out') return 'Claude usage: sign in to Claude Code'
  if (state.value === 'offline') return 'Claude usage: unavailable'
  const current = session.value
  if (!current) return 'Claude usage'
  const left = formatResetIn(current.resetsAt, now.value)
  return `Claude session at ${percentLabel(current)}${left ? `, resets in ${left}` : ''}`
})

function close(): void {
  open.value = false
}
</script>

<template>
  <div
    class="relative"
    data-testid="claude-usage"
    @mouseenter="open = true"
    @mouseleave="close"
    @focusin="open = true"
    @focusout="close"
  >
    <button
      type="button"
      class="focus-ring flex h-8 items-center gap-2 rounded-[var(--radius-control)] border border-border-strong bg-surface-raised px-2.5 text-[12px] text-text-subtle transition-colors hover:border-accent hover:bg-surface-hover"
      :aria-label="triggerLabel"
      :aria-expanded="open"
      @click="open = !open"
      @keydown.esc="close"
    >
      <template v-if="state === 'ok' && session">
        <span class="relative grid size-[22px] place-items-center" aria-hidden="true">
          <svg class="size-[22px] -rotate-90" viewBox="0 0 22 22" fill="none">
            <circle cx="11" cy="11" :r="RING_RADIUS" class="stroke-border" stroke-width="2.5" />
            <circle
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
          </svg>
        </span>
        <span class="font-mono tabular-nums" :class="TEXT[severity]">
          {{ percentLabel(session) }}
        </span>
        <span class="hidden font-mono text-[11px] text-text-subtle sm:inline">
          {{ formatResetIn(session.resetsAt, now) }}
        </span>
      </template>

      <template v-else-if="state === 'signed-out'">
        <span class="size-1.5 rounded-full bg-text-subtle" aria-hidden="true" />
        <span>Sign in</span>
      </template>

      <template v-else>
        <span class="size-1.5 rounded-full bg-border-strong" aria-hidden="true" />
        <span>Usage &mdash;</span>
      </template>
    </button>

    <div
      v-if="open"
      class="rise absolute right-0 top-full z-(--z-overlay) mt-2 w-[264px] rounded-[var(--radius-card)] border border-border-strong bg-surface-raised p-3 shadow-modal"
      role="group"
      aria-label="Claude usage"
    >
      <p class="eyebrow mb-2.5 flex items-center gap-1.5">
        <span class="h-2.5 w-[3px] shrink-0 rounded-full bg-anchor" aria-hidden="true" />
        Claude usage
      </p>

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

      <p v-else-if="state === 'signed-out'" class="text-[12px] leading-relaxed text-text-subtle">
        Sign in to Claude Code to see session and weekly usage here.
      </p>

      <p v-else class="text-[12px] leading-relaxed text-text-subtle">
        Anthropic could not be reached. The last known figures show when the connection is back.
      </p>
    </div>
  </div>
</template>
