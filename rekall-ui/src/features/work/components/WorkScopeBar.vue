<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { useRouter } from 'vue-router'
import ProjectScopeMeta from '@/features/catalog/components/ProjectScopeMeta.vue'
import ScopeBar from '@/features/catalog/components/ScopeBar.vue'
import ScopeSwitcher from '@/features/catalog/components/ScopeSwitcher.vue'
import type { ScopeTarget } from '@/features/catalog/model/scope-target'
import { useConsoleStore } from '@/stores/console.store'
import type { ProjectId } from '@/model/branded'

const store = useConsoleStore()
const router = useRouter()
const { scopedProject, scopedTasks, scopeCompany, scopeProject } = storeToRefs(store)

const taskTotal = computed(() => `${scopedTasks.value.length} ${scopedTasks.value.length === 1 ? 'task' : 'tasks'}`)

function narrowTo(target: ScopeTarget): void {
  if (target.kind === 'all') store.setScope(null, null)
  else if (target.kind === 'company') store.setScope(target.companyId, null)
  else store.setScope(target.companyId, target.projectId)
}

function openProjectPage(projectId: ProjectId): void {
  void router.push(`/projects/${projectId}/tasks`)
}
</script>

<template>
  <ScopeBar data-testid="work-toolbar">
    <template #switcher>
      <ScopeSwitcher mode="work" :company-id="scopeCompany" :project-id="scopeProject" @select="narrowTo" @open-page="openProjectPage" />
    </template>
    <template #meta>
      <ProjectScopeMeta v-if="scopedProject" :project="scopedProject" />
      <span v-else data-testid="scope-task-total">{{ taskTotal }}</span>
    </template>
  </ScopeBar>
</template>
