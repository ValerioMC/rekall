<script setup lang="ts">
/** One big number with what it counts and a line of context. */
defineProps<{ label: string; value: string; detail: string; testid: string }>()
</script>

<template>
  <div class="flex min-w-0 flex-col gap-0.5">
    <span class="eyebrow">{{ label }}</span>
    <span class="font-mono text-display text-text" :data-testid="testid">{{ value }}</span>
    <span class="text-caption text-text-subtle">{{ detail }}</span>
  </div>
</template>
