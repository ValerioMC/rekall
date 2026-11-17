<script setup lang="ts">
/**
 * A titled card of a form grid: panels sit side by side and share the window, so a settings screen
 * fills it instead of standing in a narrow column. Without a title the body takes its own padding.
 */
defineProps<{ title?: string; description?: string }>()
</script>

<template>
  <section class="min-w-0 overflow-hidden rounded-card border border-border bg-surface shadow-[var(--shadow-raised)]" data-testid="form-panel">
    <header v-if="title" class="flex items-baseline gap-3 border-b border-border px-5 py-3">
      <h2 class="text-title font-semibold tracking-[-0.01em] text-text">{{ title }}</h2>
      <p v-if="description" class="text-small text-text-subtle">{{ description }}</p>
    </header>
    <div :class="title ? '' : 'p-5'">
      <slot />
    </div>
  </section>
</template>
