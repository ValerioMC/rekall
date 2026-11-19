<script setup lang="ts">
import { computed, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useRoute, useRouter } from 'vue-router'
import ScopeSwitcher from '@/features/catalog/components/ScopeSwitcher.vue'
import type { ScopeTarget } from '@/features/catalog/model/scope-target'
import { useRecentProjectsStore } from '@/features/catalog/store/recentProjects.store'
import { useWorkStore } from '@/features/work/store/work.store'
import { useConsoleStore } from '@/stores/console.store'

/**
 * The project switcher on every screen. It names the project in view (a project page, the open
 * task's, or Work's scope) and remembers it as recent, so the list leads with where the reader
 * has been. A choice keeps a project page on its tab and otherwise narrows Work to the project.
 */
const store = useConsoleStore()
const work = useWorkStore()
const recent = useRecentProjectsStore()
const route = useRoute()
const router = useRouter()
const { selectedTask, projects } = storeToRefs(store)

const projectInView = computed(() => {
  const projectId = selectedTask.value?.projectId ?? work.projectInView
  return projects.value.find((project) => project.id === projectId) ?? null
})

watch(
  () => projectInView.value?.id ?? null,
  (projectId) => {
    if (projectId) recent.remember(projectId)
  },
  { immediate: true }
)

function open(target: ScopeTarget): void {
  if (target.kind !== 'project') return
  if (route.name === 'project-detail') {
    const tab = typeof route.params.tab === 'string' ? `/${route.params.tab}` : ''
    void router.push(`/projects/${target.projectId}${tab}`)
    return
  }
  store.setScope(target.companyId, target.projectId)
  void router.push('/')
}
</script>

<template>
  <ScopeSwitcher
    variant="header"
    mode="project"
    :company-id="projectInView?.companyId ?? null"
    :project-id="projectInView?.id ?? null"
    data-testid="header-project-switch"
    @select="open"
  />
</template>
