<script setup lang="ts">
import { computed } from 'vue'
import MacWindowControls from '@/components/console/MacWindowControls.vue'
import WindowsWindowControls from '@/components/console/WindowsWindowControls.vue'
import { desktopPlatform } from '@/common/native/desktop'

const CONTROLS = { macos: MacWindowControls, windows: WindowsWindowControls } as const

const controls = computed(() => {
  const platform = desktopPlatform()
  return platform === 'macos' || platform === 'windows' ? CONTROLS[platform] : null
})
</script>

<!-- The window buttons of the platform the app is built for. Where the window keeps its native
     frame (a browser tab, Linux) nothing is drawn. -->
<template>
  <component :is="controls" v-if="controls" />
</template>
