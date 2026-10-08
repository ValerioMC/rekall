<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import AppNavGroup from '@/components/console/AppNavGroup.vue'
import { NAV_ENTRIES, isScreenActive } from '@/model/nav-tab'

const route = useRoute()
const routeName = computed(() => (typeof route.name === 'string' ? route.name : null))
</script>

<template>
  <nav class="flex shrink-0 self-stretch" aria-label="Screen">
    <template
      v-for="entry in NAV_ENTRIES"
      :key="entry.kind === 'screen' ? entry.screen.to : entry.group.label"
    >
      <RouterLink
        v-if="entry.kind === 'screen'"
        :to="entry.screen.to"
        :replace="isScreenActive(entry.screen, routeName)"
        :title="entry.screen.hint"
        data-testid="nav-switcher-tab"
        class="nav-tab focus-ring max-[1099px]:px-2.5"
        :class="isScreenActive(entry.screen, routeName) && 'nav-tab-current'"
        :aria-current="isScreenActive(entry.screen, routeName) ? 'page' : undefined"
      >
        {{ entry.screen.label }}
      </RouterLink>
      <AppNavGroup v-else :group="entry.group" />
    </template>
  </nav>
</template>
