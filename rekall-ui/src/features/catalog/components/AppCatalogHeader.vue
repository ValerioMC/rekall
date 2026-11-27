<script setup lang="ts">
withDefaults(defineProps<{ title: string; compact?: boolean }>(), { compact: false })
</script>

<template>
  <Teleport to="#app-header-extra">
    <div class="flex min-w-0 items-center gap-3" :class="compact ? 'max-sm:hidden' : ''">
      <span class="h-5 w-px shrink-0 bg-border-strong" aria-hidden="true" />
      <h1 class="flex min-w-0 items-center gap-2 text-title font-semibold tracking-[-0.01em] text-text">
        <span class="h-3 w-[3px] shrink-0 rounded-full bg-accent" aria-hidden="true" />
        <span class="truncate">{{ title }}</span>
      </h1>
      <slot name="title-suffix" />
    </div>
  </Teleport>
  <Teleport to="#app-header-actions">
    <div class="flex min-w-0 items-center justify-end gap-3">
      <slot name="actions" />
    </div>
  </Teleport>
</template>
