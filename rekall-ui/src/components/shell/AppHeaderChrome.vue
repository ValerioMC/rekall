<script setup lang="ts">
import HeaderSessionProvider from '@/components/shell/HeaderSessionProvider.vue'
import HeaderNavigation from '@/components/shell/HeaderNavigation.vue'
import WindowsWindowControls from '@/components/console/WindowsWindowControls.vue'
import { desktopPlatform, isDesktopApp } from '@/common/native/desktop'

const isDesktop = isDesktopApp()
const platform = desktopPlatform()
const drawsWindowButtons = platform === 'windows'
const clearsTrafficLights = platform === 'macos'
</script>

<template>
  <header
    :data-tauri-drag-region="isDesktop ? 'deep' : undefined"
    class="@container sticky top-0 bg-canvas z-(--z-sticky) flex h-(--spacing-header) shrink-0 items-center border-b border-border"
  >
    <div
      class="app-header-grid grid h-full min-w-0 flex-1 items-center gap-4 px-3"
    >
      <div class="flex min-w-0 items-center gap-4 self-stretch">
        <div v-if="clearsTrafficLights" class="w-16 shrink-0 self-stretch" aria-hidden="true"></div>
        <HeaderNavigation />
      </div>

      <div id="app-header-extra" class="contents"></div>

      <div class="@container col-start-3 row-start-1 flex min-w-0 items-center justify-end gap-1.5">
        <HeaderSessionProvider />
        <div id="app-header-actions" class="contents"></div>
      </div>
    </div>
    <WindowsWindowControls v-if="drawsWindowButtons" />
  </header>
</template>
