<script setup lang="ts">
import { useConsoleStore } from '@/stores/console.store'

/**
 * The Rekall mark at the header's left edge. Pressing it opens Work on All work: every company's
 * projects with the tasks of each, whatever scope was left on.
 */
const consoleStore = useConsoleStore()

function showAllWork(): void {
  consoleStore.setScope(null, null)
}
</script>

<template>
  <RouterLink
    to="/"
    class="focus-ring grid size-8 shrink-0 cursor-pointer place-items-center rounded-control transition-colors hover:bg-surface-hover"
    aria-label="All work"
    title="All work"
    data-testid="all-work-link"
    @click="showAllWork"
  >
    <svg class="size-6" viewBox="0 0 24 24" aria-hidden="true">
      <defs>
        <radialGradient id="all-work-mark" cx="35%" cy="30%" r="75%">
          <stop offset="0" stop-color="var(--color-accent-strong)" />
          <stop offset="0.45" stop-color="var(--color-accent)" />
          <stop offset="1" stop-color="var(--color-accent-deep)" />
        </radialGradient>
      </defs>
      <circle cx="12" cy="12" r="12" fill="var(--color-accent-deep)" />
      <circle cx="12" cy="12" r="11" fill="var(--color-canvas)" />
      <circle cx="12" cy="12" r="7" fill="url(#all-work-mark)" />
    </svg>
  </RouterLink>
</template>
