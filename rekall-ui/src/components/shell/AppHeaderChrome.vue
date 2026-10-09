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
     48px row on a four-cell grid: the mark and history, the screen tabs, a centre cell, a right
     cell. The tabs and the two outer cells never depend on the screen, so the tabs sit in the same
     place on every one. The slot is `display: contents`: a screen teleports exactly two children,
     which land in the centre and right cells. The two flexible cells grow equally and keep the
     content's own width as their floor. -->
<template>
  <header
    :data-tauri-drag-region="isDesktop ? true : undefined"
    class="glass sticky top-0 z-(--z-sticky) grid h-(--spacing-header) shrink-0 grid-cols-[auto_minmax(min-content,1fr)_minmax(0,620px)_minmax(min-content,1fr)] items-center gap-4 border-b border-border px-3"
  >
    <div class="flex items-center gap-4">
      <WindowControls v-if="isDesktop" />
      <div class="flex shrink-0 items-center gap-2.5">
        <AppLogo :size="24" class="halo rounded-key" />
        <HistoryArrows />
      </div>
    </div>

    <div class="flex justify-center self-stretch">
      <AppNavSwitcher />
    </div>

    <div id="app-header-extra" class="contents"></div>
  </header>
</template>
