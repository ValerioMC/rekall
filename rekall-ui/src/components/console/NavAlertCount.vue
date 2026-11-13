<script setup lang="ts">
import type { NavAlertTone } from '@/model/nav-tab'

/** A menu alert: a count that needs the reader, in the amber of "act" or the danger tone, read out in words. */
withDefaults(defineProps<{ count: number; label: string; tone?: NavAlertTone }>(), { tone: 'danger' })
</script>

<template>
  <span
    class="inline-flex h-4 min-w-4 items-center justify-center rounded-full px-1 text-micro font-semibold leading-none tabular-nums"
    :class="tone === 'danger' ? 'bg-danger text-canvas' : 'bg-accent text-accent-ink'"
    :aria-label="label"
    :title="label"
    data-testid="nav-alert-count"
  >
    {{ count > 99 ? '99+' : count }}
  </span>
</template>
