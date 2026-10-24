<script setup lang="ts">
import type { ProjectTab } from '@/features/project/model/project-page'

/**
 * The project's pages as one row of tabs: what is written about it, what it is made of and what
 * it reaches, then its settings. A count says how many a page holds; a warning marks one that
 * needs attention.
 */
defineProps<{
  selected: ProjectTab
  counts: Partial<Readonly<Record<ProjectTab, number>>>
  warnings: Partial<Readonly<Record<ProjectTab, string>>>
}>()

const emit = defineEmits<{ select: [tab: ProjectTab] }>()

interface TabEntry {
  readonly tab: ProjectTab
  readonly label: string
}

const GROUPS: readonly (readonly TabEntry[])[] = [
  [
    { tab: 'blueprint', label: 'Blueprint' },
    { tab: 'sections', label: 'Sections' },
    { tab: 'graphs', label: 'Deployment graphs' }
  ],
  [
    { tab: 'components', label: 'Components' },
    { tab: 'connections', label: 'Connections' }
  ],
  [{ tab: 'settings', label: 'Settings' }]
]
</script>

<template>
  <nav class="tabs flex items-stretch gap-1 overflow-x-auto" aria-label="Project pages" data-testid="project-tabs">
    <template v-for="(group, index) in GROUPS" :key="index">
      <span v-if="index > 0" class="mx-2 my-2.5 w-px shrink-0 bg-border" :class="{ 'ml-auto': index === GROUPS.length - 1 }" aria-hidden="true" />
      <button
        v-for="entry in group"
        :key="entry.tab"
        type="button"
        class="tab focus-ring relative flex shrink-0 cursor-pointer items-center gap-1.5 rounded-control px-3 text-small font-medium"
        :class="selected === entry.tab ? 'text-text' : 'text-text-muted hover:text-text'"
        :aria-current="selected === entry.tab ? 'page' : undefined"
        :data-testid="`tab-${entry.tab}`"
        @click="emit('select', entry.tab)"
      >
        {{ entry.label }}
        <span
          v-if="counts[entry.tab]"
          class="font-mono text-micro tabular-nums text-text-subtle"
          :data-testid="`tab-${entry.tab}-count`"
          >{{ counts[entry.tab] }}</span
        >
        <span
          v-if="warnings[entry.tab]"
          class="size-1.5 rounded-full bg-warn"
          :title="warnings[entry.tab]"
          :data-testid="`tab-${entry.tab}-warning`"
        >
          <span class="sr-only">{{ warnings[entry.tab] }}</span>
        </span>
      </button>
    </template>
  </nav>
</template>

<style scoped>
.tabs {
  height: 40px;
  scrollbar-width: none;
}

.tab {
  transition: color var(--motion-quick) ease;
}

/* The page on screen sits on the bar's rule, the way the app header marks its own place. */
.tab[aria-current='page']::after {
  content: '';
  position: absolute;
  inset: auto 10px 0;
  height: 2px;
  border-radius: 2px 2px 0 0;
  background: var(--color-accent);
}
</style>
