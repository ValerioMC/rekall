<script setup lang="ts">
import AppNavMenu from '@/components/console/AppNavMenu.vue'
import HeaderProjectSwitch from '@/components/shell/HeaderProjectSwitch.vue'
import HistoryArrows from '@/components/shell/HistoryArrows.vue'
</script>

<template>
  <div class="@container flex min-w-0 flex-1 items-center gap-2 self-stretch" data-testid="header-navigation">
    <HistoryArrows />
    <AppNavMenu />
    <HeaderProjectSwitch />
  </div>
</template>
