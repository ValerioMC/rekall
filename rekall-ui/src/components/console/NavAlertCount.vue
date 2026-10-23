<script setup lang="ts">
/** A tab's alert: a count that needs the reader, drawn in the danger tone and read out in words. */
defineProps<{ count: number; label: string }>()
</script>

<template>
  <span
    class="inline-flex h-4 min-w-4 items-center justify-center rounded-full bg-danger px-1 text-micro font-semibold leading-none text-canvas tabular-nums"
    :aria-label="label"
    :title="label"
    data-testid="nav-alert-count"
  >
    {{ count > 99 ? '99+' : count }}
  </span>
</template>
