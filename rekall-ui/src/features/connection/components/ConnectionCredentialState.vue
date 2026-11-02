<script setup lang="ts">
import { computed } from 'vue'
import { relativeTime } from '@/common/format/relative-time'

/**
 * A credential's state as a shape and a word, never colour alone: a filled tick when stored, a
 * warning triangle when a check failed, an empty ring when there is none.
 */
const props = withDefaults(defineProps<{ credentialSetAt: string | null; checkFailed?: boolean; detailed?: boolean }>(), {
  checkFailed: false,
  detailed: false
})

type State = 'failed' | 'stored' | 'missing'

const state = computed<State>(() => {
  if (!props.credentialSetAt) return 'missing'
  return props.checkFailed ? 'failed' : 'stored'
})

const text = computed(() => {
  if (state.value === 'missing') return 'No credential'
  if (state.value === 'failed') return 'Check failed'
  return props.detailed && props.credentialSetAt ? `Stored ${relativeTime(props.credentialSetAt)}` : 'Stored'
})

const tone = computed(() => ({ failed: 'text-danger', stored: 'text-safe', missing: 'text-warn' })[state.value])
</script>

<template>
  <span class="inline-flex items-center gap-1.5 text-caption font-medium" :class="tone" :data-state="state">
    <svg viewBox="0 0 16 16" class="size-3.5 shrink-0" fill="none" aria-hidden="true">
      <template v-if="state === 'stored'">
        <circle cx="8" cy="8" r="6.5" fill="currentColor" />
        <path d="M5 8.2l2 2L11 6" stroke="var(--color-canvas)" stroke-width="1.6" stroke-linecap="round" />
      </template>
      <template v-else-if="state === 'failed'">
        <path d="M8 1.8l6.4 11.4H1.6L8 1.8z" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round" />
        <path d="M8 6.5v3M8 11.3v.1" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
      </template>
      <circle v-else cx="8" cy="8" r="6" stroke="currentColor" stroke-width="1.4" stroke-dasharray="2.4 2" />
    </svg>
    {{ text }}
  </span>
</template>
