<script setup lang="ts">
import { closeWindow, minimizeWindow, toggleMaximizeWindow } from '@/common/native/desktop'
</script>

<!-- The window draws no native titlebar buttons (`decorations(false)` in main.rs): these three
     replace them, styled like the rest of the bar's icon buttons rather than the system's own. -->
<template>
  <div class="flex shrink-0 items-center gap-1.5 pr-2.5" data-testid="window-controls">
    <button
      type="button"
      aria-label="Close"
      title="Close"
      class="focus-ring grid size-6 place-items-center rounded-[var(--radius-control)] text-text-subtle transition-colors hover:bg-danger-soft hover:text-danger"
      @click="closeWindow"
    >
      <svg class="size-3" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <path d="M5 5l14 14M19 5L5 19" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
      </svg>
    </button>
    <button
      type="button"
      aria-label="Minimize"
      title="Minimize"
      class="focus-ring grid size-6 place-items-center rounded-[var(--radius-control)] text-text-subtle transition-colors hover:bg-surface-hover hover:text-text"
      @click="minimizeWindow"
    >
      <svg class="size-3" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <path d="M5 12h14" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
      </svg>
    </button>
    <button
      type="button"
      aria-label="Maximize"
      title="Maximize"
      class="focus-ring grid size-6 place-items-center rounded-[var(--radius-control)] text-text-subtle transition-colors hover:bg-surface-hover hover:text-text"
      @click="toggleMaximizeWindow"
    >
      <svg class="size-3" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <rect x="5" y="5" width="14" height="14" rx="1.5" stroke="currentColor" stroke-width="2" />
      </svg>
    </button>
  </div>
</template>
