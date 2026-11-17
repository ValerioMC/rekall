<script setup lang="ts">
import { computed } from 'vue'
import { usePreferencesStore } from '@/features/settings/store/preferences.store'
import { columnMaxWidth } from '@/features/task/model/task-width'

/** A document's column on a project page: as wide as the page width the person chose, centred in the room it has. */
const preferences = usePreferencesStore()
const maxWidth = computed(() => columnMaxWidth(preferences.taskWidth))
</script>

<template>
  <div class="mx-auto w-full" :style="{ maxWidth }" data-testid="reading-column">
    <slot />
  </div>
</template>
