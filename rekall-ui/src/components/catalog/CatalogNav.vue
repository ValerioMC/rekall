<script setup lang="ts">
import { useRoute } from 'vue-router'

/**
 * What every catalog page opens with: the way back to the console, and the switch between the
 * two lists that live beside it.
 *
 * The pill toggle is the same shape as the tasks/notes switch in `NavigatorPane` — one control,
 * reused rather than re-invented, so "here are two related lists, pick one" reads the same way
 * everywhere it appears.
 */
const route = useRoute()

const TABS = [
  { to: '/projects', label: 'Projects', match: 'projects' },
  { to: '/companies', label: 'Companies', match: 'companies' }
] as const
</script>

<template>
  <div class="flex shrink-0 items-center gap-3">
    <RouterLink
      to="/"
      data-testid="back-to-console"
      class="focus-ring grid size-8 shrink-0 place-items-center rounded-[var(--radius-control)] border border-border-strong bg-surface-raised text-text-subtle transition-colors hover:border-accent hover:bg-surface-hover hover:text-text"
      aria-label="Back to console"
      title="Back to console"
    >
      <svg class="size-4" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <path
          d="M15 6l-6 6 6 6"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </RouterLink>

    <div class="flex gap-0.5 rounded-[8px] bg-canvas p-0.5" role="group" aria-label="Catalog">
      <RouterLink
        v-for="tab in TABS"
        :key="tab.to"
        :to="tab.to"
        class="focus-ring flex h-7 items-center justify-center rounded-[6px] px-3 text-[12px] transition-colors"
        :class="
          route.name === tab.match
            ? 'bg-surface-raised text-text shadow-[0_1px_2px_rgb(0_0_0/0.4)]'
            : 'text-text-subtle hover:text-text'
        "
        :aria-current="route.name === tab.match"
      >
        {{ tab.label }}
      </RouterLink>
    </div>
  </div>
</template>
