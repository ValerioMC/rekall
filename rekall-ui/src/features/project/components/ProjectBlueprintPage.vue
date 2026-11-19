<script setup lang="ts">
import { computed } from 'vue'
import ProjectBlueprintPane from '@/features/project/components/ProjectBlueprintPane.vue'
import ProjectContextPanel from '@/features/project/components/ProjectContextPanel.vue'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import { useProjectComponentsStore } from '@/features/project/store/projectComponents.store'
import { useProjectSectionsStore } from '@/features/project/store/projectSections.store'
import type { Project } from '@/features/catalog/model/catalog'

/** The blueprint beside what a session is handed from the project, so its weight is seen where it is written. */
const props = defineProps<{ project: Project }>()

const components = useProjectComponentsStore()
const connections = useConnectionsStore()
const sections = useProjectSectionsStore()

const contextInput = computed(() => ({
  blueprint: props.project.blueprintMarkdown,
  components: components.componentsOf(props.project.id),
  connections: connections.projectConnectionsOf(props.project.id),
  sections: sections.sectionsOf(props.project.id)
}))
</script>

<template>
  <div class="grid h-full min-h-0 grid-cols-1 gap-4 lg:grid-cols-[minmax(0,1fr)_340px]" data-testid="blueprint-page">
    <ProjectBlueprintPane :project="project" />
    <ProjectContextPanel class="hidden lg:flex" :input="contextInput" />
  </div>
</template>
