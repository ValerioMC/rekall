<script setup lang="ts">
defineProps<{ title: string; description?: string }>()
</script>

<template>
  <div
    data-testid="empty-state"
    class="flex flex-col items-center justify-center rounded-[var(--radius-card)] border border-dashed border-border px-6 py-16 text-center"
  >
    <div class="mb-3 flex size-10 items-center justify-center rounded-xl bg-surface-raised text-text-subtle">
      <slot name="icon">
        <svg viewBox="0 0 20 20" fill="none" class="size-5" aria-hidden="true">
          <rect x="3" y="4" width="14" height="12" rx="2" stroke="currentColor" stroke-width="1.4" />
          <path d="M3 8h14" stroke="currentColor" stroke-width="1.4" />
        </svg>
      </slot>
    </div>
    <h3 class="text-[15px] font-semibold text-text">{{ title }}</h3>
    <p v-if="description" class="mt-1 max-w-sm text-[13px] text-text-subtle">{{ description }}</p>
    <div class="mt-4"><slot /></div>
  </div>
</template>
