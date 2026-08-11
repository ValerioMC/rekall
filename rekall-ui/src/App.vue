<script setup lang="ts">
import AppSidebar from '@/components/shared/AppSidebar.vue'
import AppToaster from '@/components/ui/AppToaster.vue'
import CommandPalette from '@/components/shared/CommandPalette.vue'
import { useSchema } from '@/composables/useSchema'

const { entities, appliedEntities, pendingCount } = useSchema()
</script>

<template>
  <div class="flex h-full">
    <!--
      Skip link. The sidebar grows with the number of entities, and without this a keyboard
      user tabs through every one of them before reaching the screen they opened.
    -->
    <a
      href="#main"
      class="focus-ring sr-only focus:not-sr-only focus:fixed focus:left-4 focus:top-4 focus:z-(--z-toast) focus:rounded-[var(--radius-control)] focus:bg-accent focus:px-4 focus:py-2 focus:text-[13px] focus:font-semibold focus:text-accent-ink"
    >
      Skip to content
    </a>

    <AppSidebar :entities="entities" :applied-entities="appliedEntities" :pending-count="pendingCount" />

    <main id="main" class="flex min-w-0 flex-1 flex-col overflow-y-auto" tabindex="-1">
      <!-- Keyed on the full path so switching entity remounts the view instead of reusing state. -->
      <RouterView :key="$route.fullPath" />
    </main>

    <CommandPalette />
    <AppToaster />
  </div>
</template>
