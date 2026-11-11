<script setup lang="ts">
import { SETTINGS_PANES, type SettingsPane } from '@/features/project/model/project-settings'

/** The settings of a project as an index beside the one open, each with what it holds. */
defineProps<{ selected: SettingsPane }>()

const emit = defineEmits<{ select: [pane: SettingsPane] }>()
</script>

<template>
  <nav class="flex min-h-0 flex-col gap-1" aria-label="Project settings" data-testid="settings-rail">
    <button
      v-for="entry in SETTINGS_PANES"
      :key="entry.pane"
      type="button"
      class="focus-ring flex w-full flex-col rounded-control py-2 pl-4 pr-3 text-left transition-colors"
      :class="selected === entry.pane ? 'selected-row' : 'hover:bg-surface-raised'"
      :aria-current="selected === entry.pane ? 'page' : undefined"
      :data-testid="`settings-rail-${entry.pane}`"
      @click="emit('select', entry.pane)"
    >
      <span class="text-body text-text">{{ entry.label }}</span>
      <span class="text-caption text-text-subtle">{{ entry.summary }}</span>
    </button>
  </nav>
</template>
