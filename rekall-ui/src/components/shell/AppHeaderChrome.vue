<script setup lang="ts">
import AppLogo from '@/components/ui/AppLogo.vue'
import AppNavSwitcher from '@/components/console/AppNavSwitcher.vue'
import HistoryArrows from '@/components/shell/HistoryArrows.vue'
import WindowControls from '@/components/console/WindowControls.vue'
import { desktopPlatform, isDesktopApp } from '@/common/native/desktop'

const isDesktop = isDesktopApp()
// Windows keeps its window buttons at the right edge, macOS at the left: the platform decides.
const controlsAtRight = desktopPlatform() === 'windows'
</script>

<!-- The one header instance for every screen: mounted once by Bootstrap, above `router-view`, so
     it never unmounts on navigation. Each screen teleports its own content (a search bar, a page
     title, its actions) into `#app-header-extra` rather than rendering a header of its own. One
     48px row on a three-cell grid (`app-header-grid`): the mark, history and screen tabs; a centre
     cell; a right cell. The sides are equal, so the centre cell is at exactly half the grid. The
     left cell never depends on the screen, so the tabs sit in the same place on every one. The
     slot is `display: contents`: a screen teleports exactly two children, which land in the
     centre and right cells. On Windows the window buttons stand outside the grid, flush with the
     right edge, so the grid is the space left of them. -->
<template>
  <header
    :data-tauri-drag-region="isDesktop ? true : undefined"
    class="glass sticky top-0 z-(--z-sticky) flex h-(--spacing-header) shrink-0 items-center border-b border-border"
  >
    <div
      :data-tauri-drag-region="isDesktop ? true : undefined"
      class="app-header-grid grid h-full min-w-0 flex-1 items-center gap-4 px-3"
    >
      <div class="flex min-w-0 items-center gap-4 self-stretch">
        <WindowControls v-if="isDesktop && !controlsAtRight" />
        <div class="flex shrink-0 items-center gap-2.5">
          <AppLogo :size="24" class="halo rounded-key" />
          <HistoryArrows />
        </div>
        <div class="@container flex min-w-0 flex-1 justify-center self-stretch">
          <AppNavSwitcher />
        </div>
      </div>

      <div id="app-header-extra" class="contents"></div>
    </div>
    <WindowControls v-if="isDesktop && controlsAtRight" />
  </header>
</template>
