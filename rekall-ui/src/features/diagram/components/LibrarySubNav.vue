<script setup lang="ts">
import SubNav from '@/components/ui/SubNav.vue'
import type { SubNavItem } from '@/components/ui/SubNav.vue'

type LibraryScreen = 'diagrams' | 'claude'

/** The two screens of the Library, the diagrams and the Claude files, as links under the page title. */
defineProps<{ active: LibraryScreen }>()

const ITEMS: readonly SubNavItem<LibraryScreen>[] = [
  { id: 'diagrams', label: 'Diagrams', to: '/diagrams' },
  { id: 'claude', label: 'Claude files', to: '/claude' }
]
</script>

<template>
  <SubNav :model-value="active" :items="ITEMS" label="Library screens" />
</template>
