<script setup lang="ts">
import AppLogo from '@/components/ui/AppLogo.vue'
import { useConsoleStore } from '@/stores/console.store'

/**
 * The Rekall icon in the header navigation. Pressing it opens the project list and clears the scope
 * left on, so no company or project narrows what the other screens show.
 */
const consoleStore = useConsoleStore()

function clearScope(): void {
  consoleStore.setScope(null, null)
}
</script>

<template>
  <RouterLink
    to="/projects"
    class="focus-ring grid size-8 shrink-0 cursor-pointer place-items-center rounded-control transition-colors hover:bg-surface-hover"
    aria-label="Projects"
    title="Projects"
    data-testid="home-link"
    @click="clearScope"
  >
    <AppLogo :size="24" aria-hidden="true" />
  </RouterLink>
</template>
