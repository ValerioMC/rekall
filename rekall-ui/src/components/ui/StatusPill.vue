<script setup lang="ts">
export type StatusPillTone = 'neutral' | 'progress' | 'review' | 'done' | 'filed' | 'running' | 'danger'

/**
 * A record's state as one word in a pill. The tone is the state's colour, never decoration;
 * `live` adds the session orb for a state that is happening now.
 */
withDefaults(defineProps<{ tone?: StatusPillTone; live?: boolean }>(), { tone: 'neutral', live: false })

const TONES: Readonly<Record<StatusPillTone, string>> = {
  neutral: 'border-border text-text-muted',
  progress: 'border-accent/35 bg-accent-soft text-accent-strong',
  review: 'border-accent bg-accent text-accent-ink',
  done: 'border-safe/30 bg-safe-soft text-safe',
  filed: 'border-filed-line bg-filed-soft text-filed',
  running: 'border-session/35 bg-session/15 text-session-strong',
  danger: 'border-danger/35 bg-danger-soft text-danger'
}
</script>

<template>
  <span
    class="inline-flex h-[22px] items-center gap-1.5 whitespace-nowrap rounded-full border px-2 text-caption font-medium"
    :class="TONES[tone]"
    :data-tone="tone"
    data-testid="status-pill"
  >
    <span v-if="live" class="size-1.5 animate-pulse rounded-full bg-session" aria-hidden="true" data-testid="status-pill-orb" />
    <slot />
  </span>
</template>
