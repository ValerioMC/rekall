<script setup lang="ts">
import AppNavMenu from '@/components/console/AppNavMenu.vue'
import AllWorkLink from '@/components/shell/AllWorkLink.vue'
import HistoryArrows from '@/components/shell/HistoryArrows.vue'
</script>

<template>
  <div class="flex min-w-0 flex-1 items-center gap-2.5 self-stretch" data-testid="header-navigation">
    <AllWorkLink />
    <AppNavMenu />
    <HistoryArrows />
  </div>
</template>
