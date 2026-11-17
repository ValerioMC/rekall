<script setup lang="ts">
/** One setting of a panel: its name and what it does on the left, its control on the right. */
defineProps<{ label: string; hint?: string; labelFor?: string }>()
</script>

<template>
  <div class="grid items-center gap-x-8 gap-y-2 border-b border-border px-5 py-3.5 last:border-b-0 min-[720px]:grid-cols-[minmax(160px,1fr)_minmax(220px,auto)]" data-testid="form-row">
    <div class="min-w-0">
      <label v-if="labelFor" :for="labelFor" class="block text-body font-medium text-text">{{ label }}</label>
      <p v-else class="text-body font-medium text-text">{{ label }}</p>
      <p v-if="hint" class="mt-0.5 text-small leading-relaxed text-text-subtle">{{ hint }}</p>
    </div>
    <div class="flex min-w-0 min-[720px]:justify-end">
      <slot />
    </div>
  </div>
</template>
