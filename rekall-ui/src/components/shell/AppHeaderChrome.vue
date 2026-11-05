<script setup lang="ts">
import HeaderNavigation from '@/components/shell/HeaderNavigation.vue'
import WindowsWindowControls from '@/components/console/WindowsWindowControls.vue'
import { desktopPlatform, isDesktopApp } from '@/common/native/desktop'

const isDesktop = isDesktopApp()
const platform = desktopPlatform()
const drawsWindowButtons = platform === 'windows'
const clearsTrafficLights = platform === 'macos'
</script>

<!-- The one header instance for every screen: mounted once by Bootstrap, above `router-view`, so
     it never unmounts on navigation. Each screen teleports its own content (a search bar, a page
     title, its actions) into `#app-header-extra` rather than rendering a header of its own. One
     48px row on a three-cell grid (`app-header-grid`): the menu key with the history arrows or the
     screen tabs; a centre cell; a right cell. The sides are equal, so the centre cell is at exactly
     half the grid. The left cell never depends on the screen, so the key sits in the same place on
     every one. The slot is `display: contents`: a screen teleports exactly two children, which land
     in the centre and right cells. On macOS the system's own window buttons float over the left
     edge, so the left cell starts clear of them. On Windows the window buttons stand outside the
     grid, flush with the right edge, so the grid is the space left of them. -->
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
        <div v-if="clearsTrafficLights" class="w-16 shrink-0 self-stretch" aria-hidden="true"></div>
        <HeaderNavigation />
      </div>

      <div id="app-header-extra" class="contents"></div>
    </div>
    <WindowsWindowControls v-if="drawsWindowButtons" />
  </header>
</template>
