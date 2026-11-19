<script setup lang="ts">
import AppLogo from '@/components/ui/AppLogo.vue'
import { useConsoleStore } from '@/stores/console.store'

/**
 * The Rekall icon at the header's left edge. Pressing it opens Work on All work: every company's
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
    <AppLogo :size="24" aria-hidden="true" />
  </RouterLink>
</template>
