<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import ConsoleHeader from '@/components/console/ConsoleHeader.vue'
import ProjectScopeMeta from '@/features/catalog/components/ProjectScopeMeta.vue'
import ScopeBar from '@/features/catalog/components/ScopeBar.vue'
import ScopeSwitcher from '@/features/catalog/components/ScopeSwitcher.vue'
import SubNav from '@/components/ui/SubNav.vue'
import type { SubNavItem } from '@/components/ui/SubNav.vue'
import ProjectDiagramsTab from '@/features/project/components/ProjectDiagramsTab.vue'
import RecordDialog from '@/features/catalog/components/RecordDialog.vue'
import TasksSection from '@/features/work/components/TasksSection.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import ProjectBlueprintPane from '@/features/project/components/ProjectBlueprintPane.vue'
import ProjectComponentsPage from '@/features/project/components/ProjectComponentsPage.vue'
import ProjectSectionsPage from '@/features/project/components/ProjectSectionsPage.vue'
import ProjectSettingsPage from '@/features/project/components/ProjectSettingsPage.vue'
import ProjectConnectionsPage from '@/features/connection/components/ProjectConnectionsPage.vue'
import ProjectGraphsPage from '@/features/graph/components/ProjectGraphsPage.vue'
import { useDeploymentGraphsStore } from '@/features/graph/store/deploymentGraphs.store'
import { useConsoleStore } from '@/stores/console.store'
import { useProjectSectionsStore } from '@/features/project/store/projectSections.store'
import { useProjectComponentsStore } from '@/features/project/store/projectComponents.store'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import { useConnectionStream } from '@/features/connection/composables/useConnectionStream'
import { useProjectSectionStream } from '@/features/project/composables/useProjectSectionStream'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useRecentProjectsStore } from '@/features/catalog/store/recentProjects.store'
import { useTaskEditor } from '@/features/catalog/composables/useTaskEditor'
import { useWorkStore } from '@/features/work/store/work.store'
import type { ScopeTarget } from '@/features/catalog/model/scope-target'
import { asProjectId } from '@/model/branded'
import { PROJECT_TABS, PROJECT_TAB_LABEL, projectTabOf, type ProjectTab } from '@/features/project/model/project-page'
import type { DeploymentGraphId, ProjectSectionId } from '@/model/branded'

const props = defineProps<{ id: string; tab?: string }>()

const store = useConsoleStore()
const router = useRouter()
const tab = computed(() => projectTabOf(props.tab))

const projectId = computed(() => asProjectId(props.id))
const project = computed(() => store.projects.find((candidate) => candidate.id === projectId.value) ?? null)
const projectTasks = computed(() =>
  store.tasks.filter((task) => task.projectId === projectId.value)
)

const sectionsStore = useProjectSectionsStore()
const graphsStore = useDeploymentGraphsStore()
const componentsStore = useProjectComponentsStore()
const connectionsStore = useConnectionsStore()
const recentProjects = useRecentProjectsStore()
const work = useWorkStore()
const taskEditor = useTaskEditor()
const { run } = useAsyncAction()

const sections = computed(() => sectionsStore.sectionsOf(projectId.value))
const graphs = computed(() => graphsStore.graphsOf(projectId.value))
const bindings = computed(() => connectionsStore.projectConnectionsOf(projectId.value))
const sectionId = ref<ProjectSectionId | null>(null)
const graphId = ref<DeploymentGraphId | null>(null)
const settingUpGraph = ref(false)

const tabItems = computed<SubNavItem<ProjectTab>[]>(() => {
  const missing = bindings.value.filter((binding) => !binding.credentialSetAt).length
  const counts: Partial<Record<ProjectTab, number>> = {
    tasks: projectTasks.value.length,
    pages: sections.value.length,
    components: componentsStore.componentsOf(projectId.value).length,
    deployment: graphs.value.length,
    connections: bindings.value.length
  }
  return PROJECT_TABS.map((id) => ({
    id,
    label: PROJECT_TAB_LABEL[id],
    to: `/projects/${props.id}/${id}`,
    ...(counts[id] ? { count: counts[id] } : {}),
    ...(id === 'connections' && missing > 0 ? { hot: true } : {})
  }))
})

useProjectSectionStream((event) => void sectionsStore.applyEvent(event.projectId))
useConnectionStream((event) => {
  if (project.value && event.companyIds.includes(project.value.companyId)) void loadConnections()
})

async function loadConnections(): Promise<void> {
  const shown = project.value
  if (!shown) return
  await run(() => Promise.all([connectionsStore.loadProject(shown.id), connectionsStore.load()]))
}

function openSection(id: ProjectSectionId | null): void {
  sectionId.value = id
}

function openGraph(id: DeploymentGraphId | null): void {
  graphId.value = id
  settingUpGraph.value = false
}

function leaveDeletedProject(): void {
  void router.push('/projects')
}

function openProject(target: ScopeTarget): void {
  if (target.kind === 'project') void router.push(`/projects/${target.projectId}/${tab.value}`)
}

watch(
  () => project.value?.id ?? null,
  () => {
    work.focusProject(project.value?.id ?? null)
    sectionId.value = null
    graphId.value = null
    settingUpGraph.value = false
    if (!project.value) return
    const id = project.value.id
    recentProjects.remember(id)
    void run(() => sectionsStore.load(id))
    void run(() => graphsStore.load(id))
    void run(() => componentsStore.load(id))
    void loadConnections()
  },
  { immediate: true }
)

onBeforeUnmount(() => work.focusProject(null))

watch(
  [sections, tab],
  () => {
    if (tab.value !== 'pages') return
    if (!sections.value.some((section) => section.id === sectionId.value)) sectionId.value = sections.value[0]?.id ?? null
  },
  { immediate: true }
)

watch([graphs, tab], () => {
  if (tab.value !== 'deployment') return
  if (!graphs.value.some((graph) => graph.id === graphId.value)) graphId.value = graphs.value[0]?.id ?? null
})
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col page-scroll bg-canvas">
    <template v-if="!store.isLoading && !project">
      <ConsoleHeader />
      <div class="page-column py-6">
        <AppEmptyState title="No project here" description="It may have been deleted, or the link is stale.">
          <RouterLink to="/projects"><AppButton variant="secondary" size="sm">Back to projects</AppButton></RouterLink>
        </AppEmptyState>
      </div>
    </template>

    <template v-else-if="!project">
      <ConsoleHeader />
      <div class="page-column flex flex-col gap-4 py-6" aria-hidden="true">
        <div class="skeleton h-32 rounded-card" />
        <div class="skeleton h-64 rounded-card" />
      </div>
    </template>

    <template v-else>
      <ConsoleHeader :anchor="project.anchor" />

      <div class="page-column flex min-h-[540px] flex-1 basis-0 flex-col" data-testid="project-workspace">
        <ScopeBar>
          <template #switcher>
            <ScopeSwitcher mode="project" :company-id="project.companyId" :project-id="project.id" @select="openProject" />
          </template>
          <template #meta><ProjectScopeMeta :project="project" /></template>
        </ScopeBar>
        <SubNav class="mt-3 shrink-0" :model-value="tab" :items="tabItems" label="Project pages" />
        <div class="grid min-h-0 flex-1 grid-rows-[minmax(0,1fr)] py-4">
          <Transition name="fade-quick" mode="out-in">
            <TasksSection v-if="tab === 'tasks'" key="tasks" :chooses-layout="false" />
            <ProjectBlueprintPane v-else-if="tab === 'blueprint'" key="blueprint" :project="project" />
            <ProjectSectionsPage
              v-else-if="tab === 'pages'"
              key="pages"
              :project="project"
              :selected-id="sectionId"
              @open="openSection"
            />
            <ProjectComponentsPage
              v-else-if="tab === 'components'"
              key="components"
              :project-id="project.id"
              :project-title="project.title"
              :repo-folder="project.repoFolder"
            />
            <ProjectDiagramsTab v-else-if="tab === 'diagrams'" key="diagrams" :project="project" />
            <ProjectGraphsPage
              v-else-if="tab === 'deployment'"
              key="deployment"
              :project="project"
              :selected-id="settingUpGraph ? null : graphId"
              :setting-up="settingUpGraph"
              @open="openGraph"
              @setup="settingUpGraph = true"
            />
            <ProjectConnectionsPage v-else-if="tab === 'connections'" key="connections" :project="project" />
            <ProjectSettingsPage v-else key="settings" :project="project" @deleted="leaveDeletedProject" />
          </Transition>
        </div>
      </div>
    </template>

    <Transition name="dialog">
      <RecordDialog v-if="taskEditor.editing.value" :draft="taskEditor.editing.value" @close="taskEditor.close()" />
    </Transition>
  </div>
</template>
