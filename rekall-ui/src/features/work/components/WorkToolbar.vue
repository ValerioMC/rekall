<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { useRouter } from 'vue-router'
import AppButton from '@/components/ui/AppButton.vue'
import SubNav, { type SubNavItem } from '@/components/ui/SubNav.vue'
import ProjectScopeMeta from '@/features/catalog/components/ProjectScopeMeta.vue'
import ScopeBar from '@/features/catalog/components/ScopeBar.vue'
import ScopeSwitcher from '@/features/catalog/components/ScopeSwitcher.vue'
import type { ScopeTarget } from '@/features/catalog/model/scope-target'
import WorkTools from '@/features/work/components/WorkTools.vue'
import { useWorkLayout } from '@/features/work/composables/useWorkLayout'
import { useConsoleStore } from '@/stores/console.store'
import { useWorkStore } from '@/features/work/store/work.store'
import { WORK_VIEWS, WORK_VIEW_LABEL, type WorkView } from '@/features/work/model/work-view'
import type { ProjectId } from '@/model/branded'

defineEmits<{ newTask: [] }>()

const store = useConsoleStore()
const work = useWorkStore()
const router = useRouter()
const { scopedProjects, scopedProject, scopedTasks, scopeCompany, scopeProject } = storeToRefs(store)
const { view, counts } = storeToRefs(work)

const layout = useWorkLayout()

const canCreate = computed(() => scopedProjects.value.length > 0)
const taskTotal = computed(() => `${scopedTasks.value.length} ${scopedTasks.value.length === 1 ? 'task' : 'tasks'}`)

const tabs = computed<SubNavItem<WorkView>[]>(() =>
  WORK_VIEWS.map((id) => ({ id, label: WORK_VIEW_LABEL[id], count: counts.value[id], hot: id === 'needs' }))
)

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
  <div data-testid="work-toolbar">
    <ScopeBar>
      <template #switcher>
        <ScopeSwitcher mode="work" :company-id="scopeCompany" :project-id="scopeProject" @select="narrowTo" @open-page="openProjectPage" />
      </template>
      <template #meta>
        <ProjectScopeMeta v-if="scopedProject" :project="scopedProject" />
        <span v-else data-testid="scope-task-total">{{ taskTotal }}</span>
      </template>
      <template #actions>
        <AppButton v-if="canCreate" variant="primary" data-testid="new-task" @click="$emit('newTask')">
          <span aria-hidden="true">+</span> New task
        </AppButton>
      </template>
    </ScopeBar>
    <SubNav v-if="layout === 'ledger'" v-model="view" class="mt-3" :items="tabs" label="Work views">
      <template #trailing><WorkTools /></template>
    </SubNav>
    <div v-else class="flex items-center justify-end gap-2 pt-3"><WorkTools /></div>
  </div>
</template>
