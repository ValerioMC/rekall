<script setup lang="ts">
import { computed } from 'vue'

type Tone = 'neutral' | 'accent' | 'safe' | 'warn' | 'danger' | 'filed'

const props = withDefaults(
  defineProps<{ tone?: Tone; mono?: boolean; dot?: boolean }>(),
  { tone: 'neutral', mono: false, dot: false }
)

const TONES: Readonly<Record<Tone, string>> = {
  neutral: 'bg-surface-raised text-text-muted ring-border',
  accent: 'bg-accent-soft text-accent ring-accent/25',
  safe: 'bg-safe-soft text-safe ring-safe/25',
  warn: 'bg-warn-soft text-warn ring-warn/25',
  danger: 'bg-danger-soft text-danger ring-danger/25',
  filed: 'bg-filed-soft text-filed ring-filed/25'
}

const DOTS: Readonly<Record<Tone, string>> = {
  neutral: 'bg-text-subtle',
  accent: 'bg-accent',
  safe: 'bg-safe',
  warn: 'bg-warn',
  danger: 'bg-danger',
  filed: 'bg-filed'
}

const classes = computed(() => [
  'inline-flex items-center gap-1.5 rounded-md px-2 py-0.5 text-[11px] font-medium ring-1 ring-inset',
  props.mono ? 'font-mono' : '',
  TONES[props.tone]
])
</script>

<template>
  <span :class="classes">
    <span v-if="props.dot" class="size-1.5 rounded-full" :class="DOTS[props.tone]" aria-hidden="true" />
    <slot />
  </span>
</template>
