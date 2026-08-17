<script setup lang="ts">
import { useRoute } from 'vue-router'

const route = useRoute()

const TABS = [
  { to: '/', label: 'Console', match: 'console' },
  { to: '/projects', label: 'Projects', match: 'projects' },
  { to: '/companies', label: 'Companies', match: 'companies' }
] as const

const isActive = (match: (typeof TABS)[number]['match']): boolean =>
  match === 'projects' ? route.name === 'projects' || route.name === 'project-detail' : route.name === match
</script>

<template>
  <div class="flex shrink-0 gap-0.5 rounded-[8px] bg-canvas p-0.5" role="group" aria-label="Screen">
    <RouterLink
      v-for="tab in TABS"
      :key="tab.to"
      :to="tab.to"
      data-testid="nav-switcher-tab"
      class="focus-ring flex h-7 items-center justify-center rounded-[6px] px-3 text-[12px] transition-colors"
      :class="
        isActive(tab.match)
          ? 'bg-surface-raised text-text shadow-[0_1px_2px_rgb(0_0_0/0.4)]'
          : 'text-text-subtle hover:text-text'
      "
      :aria-current="isActive(tab.match)"
    >
      {{ tab.label }}
    </RouterLink>
  </div>
</template>
