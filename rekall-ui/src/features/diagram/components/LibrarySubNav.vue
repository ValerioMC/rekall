<script setup lang="ts">
import { computed } from 'vue'
import SubNav from '@/components/ui/SubNav.vue'
import type { SubNavItem } from '@/components/ui/SubNav.vue'

type LibraryScreen = 'diagrams' | 'exports' | 'claude'

/** The screens of the Library, the diagrams, the exports and the Claude files, as links under the page title. The Exports tab carries its count once it is known. */
const props = withDefaults(defineProps<{ active: LibraryScreen; exportsCount?: number }>(), { exportsCount: undefined })

const items = computed<readonly SubNavItem<LibraryScreen>[]>(() => [
  { id: 'diagrams', label: 'Diagrams', to: '/diagrams' },
  { id: 'exports', label: 'Exports', to: '/exports', ...(props.exportsCount === undefined ? {} : { count: props.exportsCount }) },
  { id: 'claude', label: 'Claude files', to: '/claude' }
])
</script>

<template>
  <SubNav :model-value="active" :items="items" label="Library screens" />
</template>
