<script setup lang="ts">
/** A run under way: what it is doing and how far it has got. */
defineProps<{ label: string; percentage: number }>()
</script>

<template>
  <div class="px-5 py-4">
    <div class="mb-2 flex items-baseline justify-between gap-4">
      <p class="text-body font-medium text-text">{{ label }}</p>
      <p class="text-body tabular-nums text-text-muted">{{ percentage }}%</p>
    </div>
    <div
      class="trough h-2 overflow-hidden"
      role="progressbar"
      :aria-label="label"
      aria-valuemin="0"
      aria-valuemax="100"
      :aria-valuenow="percentage"
    >
      <div v-if="percentage > 0" class="tube-gold h-full rounded-full transition-[width] duration-(--motion-slow)" :style="{ width: `${percentage}%` }" />
    </div>
    <p class="mt-2 text-small text-text-subtle">You can keep working. Rekall tells you when it ends.</p>
  </div>
</template>
