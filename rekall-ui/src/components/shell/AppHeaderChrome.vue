<script setup lang="ts">
import AppLogo from '@/components/ui/AppLogo.vue'
import AppNavSwitcher from '@/components/console/AppNavSwitcher.vue'
import HistoryArrows from '@/components/shell/HistoryArrows.vue'
import WindowControls from '@/components/console/WindowControls.vue'
import { isDesktopApp } from '@/common/native/desktop'

const isDesktop = isDesktopApp()
</script>

<!-- The one header instance for every screen: mounted once by Bootstrap, above `router-view`, so
     it never unmounts on navigation. Each screen teleports its own content (a search bar, a page
     title, its actions) into `#app-header-extra` rather than rendering a header of its own. One
     48px row: the mark, history, the screen tabs, then whatever the screen puts there. -->
<template>
  <header
    :data-tauri-drag-region="isDesktop ? true : undefined"
    class="glass sticky top-0 z-(--z-sticky) flex h-(--spacing-header) shrink-0 items-center gap-4 border-b border-border px-3"
  >
    <WindowControls v-if="isDesktop" />

    <div class="flex shrink-0 items-center gap-2.5">
      <AppLogo :size="24" class="halo rounded-key" />
      <HistoryArrows />
    </div>

    <AppNavSwitcher />

    <div id="app-header-extra" class="flex min-w-0 flex-1 items-center gap-3"></div>
  </header>
</template>
