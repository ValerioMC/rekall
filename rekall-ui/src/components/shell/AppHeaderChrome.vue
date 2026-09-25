<script setup lang="ts">
import AppLogo from '@/components/ui/AppLogo.vue'
import AppNavSwitcher from '@/components/console/AppNavSwitcher.vue'
import WindowControls from '@/components/console/WindowControls.vue'
import { isDesktopApp } from '@/common/native/desktop'

const isDesktop = isDesktopApp()
</script>

<!-- The one header instance for every screen: mounted once by Bootstrap, above `router-view`, so
     it never unmounts on navigation. Each screen teleports its own content (a search bar, a page
     title, its actions) into `#app-header-extra` below rather than rendering a header of its own,
     which is what used to make the logo/tabs block flicker and drift between screens. -->
<template>
  <header
    :data-tauri-drag-region="isDesktop ? true : undefined"
    class="glass sticky top-0 z-(--z-sticky) flex h-(--spacing-header) shrink-0 items-center gap-3.5 border-b border-border px-4"
  >
    <WindowControls v-if="isDesktop" />

    <div class="flex w-(--spacing-nav) shrink-0 items-center gap-2.5 pr-2.5">
      <AppLogo :size="32" class="halo rounded-[7px]" />
      <span class="min-w-0">
        <span class="block text-[14.5px] font-semibold leading-tight tracking-[-0.015em] text-text">
          Rekall
        </span>
        <span class="block font-mono text-[10px] leading-tight text-text-subtle">
          context, anchored
        </span>
      </span>
    </div>

    <AppNavSwitcher />

    <div id="app-header-extra" class="flex min-w-0 flex-1 items-center gap-3.5"></div>
  </header>
</template>
