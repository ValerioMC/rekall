<script setup lang="ts">
import SubNav from '@/components/ui/SubNav.vue'
import type { SubNavItem } from '@/components/ui/SubNav.vue'

type TimeScreen = 'report' | 'calendar'

/** The two screens of Time, the report and the calendar, as links under the page title. */
defineProps<{ active: TimeScreen }>()

const ITEMS: readonly SubNavItem<TimeScreen>[] = [
  { id: 'report', label: 'Report', to: '/report' },
  { id: 'calendar', label: 'Calendar', to: '/calendar' }
]
</script>

<template>
  <SubNav :model-value="active" :items="ITEMS" label="Time screens" />
</template>
