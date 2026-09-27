<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import AppNavGroup from '@/components/console/AppNavGroup.vue'
import { NAV_ENTRIES, isScreenActive } from '@/model/nav-tab'

const route = useRoute()
const routeName = computed(() => (typeof route.name === 'string' ? route.name : null))
</script>

<template>
  <div class="flex shrink-0 gap-0.5 rounded-[8px] bg-canvas p-0.5" role="group" aria-label="Screen">
    <template
      v-for="entry in NAV_ENTRIES"
      :key="entry.kind === 'screen' ? entry.screen.to : entry.group.label"
    >
      <RouterLink
        v-if="entry.kind === 'screen'"
        :to="entry.screen.to"
        :title="entry.screen.hint"
        data-testid="nav-switcher-tab"
        class="focus-ring flex h-7 items-center justify-center rounded-[6px] px-3 text-[12px] transition-colors"
        :class="
          isScreenActive(entry.screen, routeName)
            ? 'bg-surface-raised text-text shadow-[0_1px_2px_rgb(0_0_0/0.4)]'
            : 'text-text-subtle hover:text-text'
        "
        :aria-current="isScreenActive(entry.screen, routeName) ? 'page' : undefined"
      >
        {{ entry.screen.label }}
      </RouterLink>
      <AppNavGroup v-else :group="entry.group" />
    </template>
  </div>
</template>
