<script setup lang="ts">
import { computed, onMounted } from 'vue'
import SubNav from '@/components/ui/SubNav.vue'
import type { SubNavItem } from '@/components/ui/SubNav.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import { PROJECTS_SECTIONS } from '@/features/catalog/model/projects-section'
import type { ProjectsSection } from '@/features/catalog/model/projects-section'
import { useConsoleStore } from '@/stores/console.store'

/** The three screens of the Projects section as sub-navigation, each with how many it holds, in the page column all three share. Its slot is the screen's own search, at the right end of the row. */
const props = defineProps<{ active: ProjectsSection }>()

const store = useConsoleStore()
const connections = useConnectionsStore()
const { run } = useAsyncAction()

onMounted(() => {
  if (props.active !== 'connections') void run(() => connections.ensureLoaded())
})

const items = computed<SubNavItem<ProjectsSection>[]>(() =>
  PROJECTS_SECTIONS.map((section) => ({
    id: section.id,
    label: section.label,
    to: section.to,
    ...countOf(section.id)
  }))
)

function countOf(id: ProjectsSection): { count?: number } {
  if (id === 'projects') return { count: store.projects.filter((project) => project.status !== 'ARCHIVED').length }
  if (id === 'companies') return { count: store.companies.length }
  return connections.loaded ? { count: connections.connections.length } : {}
}
</script>

<template>
  <div class="page-column shrink-0 pt-4">
    <SubNav :model-value="active" :items="items" label="Projects section">
      <template v-if="$slots.default" #trailing>
        <slot />
      </template>
    </SubNav>
  </div>
</template>
