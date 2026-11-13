<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import AppCatalogHeader from '@/features/catalog/components/AppCatalogHeader.vue'
import CopyAnchorButton from '@/components/console/CopyAnchorButton.vue'
import SubNav from '@/components/ui/SubNav.vue'
import type { SubNavItem } from '@/components/ui/SubNav.vue'
import ProjectDiagramsTab from '@/features/project/components/ProjectDiagramsTab.vue'
import ProjectTasksTab from '@/features/project/components/ProjectTasksTab.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import ProjectBlueprintPane from '@/features/project/components/ProjectBlueprintPane.vue'
import ProjectComponentsPage from '@/features/project/components/ProjectComponentsPage.vue'
import ProjectSectionsPage from '@/features/project/components/ProjectSectionsPage.vue'
import ProjectSettingsPage from '@/features/project/components/ProjectSettingsPage.vue'
import ProjectConnectionsPage from '@/features/connection/components/ProjectConnectionsPage.vue'
import ProjectGraphsPage from '@/features/graph/components/ProjectGraphsPage.vue'
import { useDeploymentGraphsStore } from '@/features/graph/store/deploymentGraphs.store'
import ProjectMark from '@/components/ui/ProjectMark.vue'
import TokenCount from '@/components/ui/TokenCount.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useProjectSectionsStore } from '@/features/project/store/projectSections.store'
import { useProjectComponentsStore } from '@/features/project/store/projectComponents.store'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import { useConnectionStream } from '@/features/connection/composables/useConnectionStream'
import { useProjectSectionStream } from '@/features/project/composables/useProjectSectionStream'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useTokenUsageStore } from '@/features/tokens/store/tokenUsage.store'
import { PROJECT_STATUS_LABEL } from '@/features/catalog/model/project-status'
import { asProjectId } from '@/model/branded'
import { PROJECT_TABS, PROJECT_TAB_LABEL, projectTabOf, type ProjectTab } from '@/features/project/model/project-page'
import type { DeploymentGraphId, ProjectSectionId, TaskId } from '@/model/branded'

const props = defineProps<{ id: string; tab?: string }>()

const store = useConsoleStore()
const router = useRouter()
const tab = computed(() => projectTabOf(props.tab))

const projectId = computed(() => asProjectId(props.id))
const project = computed(() => store.projects.find((candidate) => candidate.id === projectId.value) ?? null)
const company = computed(
  () => store.companies.find((candidate) => candidate.id === project.value?.companyId) ?? null
)
const projectTasks = computed(() =>
  store.tasks.filter((task) => task.projectId === projectId.value)
)

const sectionsStore = useProjectSectionsStore()
const graphsStore = useDeploymentGraphsStore()
const componentsStore = useProjectComponentsStore()
const connectionsStore = useConnectionsStore()
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

const tokenUsage = useTokenUsageStore()
const tokens = computed(() => tokenUsage.ofProject(projectId.value, store.tasks))

function leaveDeletedProject(): void {
  void router.push('/projects')
}

function openTask(taskId: TaskId): void {
  void router.push(`/task/${taskId}`)
}

const justSaved = ref(false)
watch(
  () => store.saveState,
  (value, previous) => {
    if (value === 'saved' && previous !== 'saved') {
      justSaved.value = false
      requestAnimationFrame(() => {
        justSaved.value = true
        setTimeout(() => (justSaved.value = false), 340)
      })
    }
  }
)

watch(
  () => project.value?.id ?? null,
  () => {
    sectionId.value = null
    graphId.value = null
    settingUpGraph.value = false
    if (!project.value) return
    const id = project.value.id
    void run(() => sectionsStore.load(id))
    void run(() => graphsStore.load(id))
    void run(() => componentsStore.load(id))
    void loadConnections()
  },
  { immediate: true }
)

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
  <div class="flex min-h-0 flex-1 flex-col overflow-y-auto bg-canvas">
    <template v-if="!store.isLoading && !project">
      <AppCatalogHeader title="Project" />
      <div class="mx-auto w-full max-w-[1800px] px-6 py-6 2xl:px-10">
        <AppEmptyState title="No project here" description="It may have been deleted, or the link is stale.">
          <RouterLink to="/projects"><AppButton variant="secondary" size="sm">Back to projects</AppButton></RouterLink>
        </AppEmptyState>
      </div>
    </template>

    <template v-else-if="!project">
      <Teleport to="#app-header-extra">
        <div class="flex min-w-0 items-center gap-3" aria-hidden="true">
          <span class="h-5 w-px shrink-0 bg-border-strong" />
          <div class="skeleton h-4 w-64" />
        </div>
        <div />
      </Teleport>
      <div class="mx-auto flex w-full max-w-[1800px] flex-col gap-4 px-6 py-6 2xl:px-10" aria-hidden="true">
        <div class="skeleton h-32 rounded-card" />
        <div class="skeleton h-64 rounded-card" />
      </div>
    </template>

    <template v-else>
      <AppCatalogHeader :title="project.title" :parent="company?.name">
        <template #leading>
          <ProjectMark :project-id="project.id" :icon="project.icon" :image-version="project.iconImageVersion" :size="24" />
        </template>
        <template #title-suffix>
          <CopyAnchorButton :anchor="project.anchor" data-testid="copy-project-anchor" />
          <TokenCount :totals="tokens" :hue-of="project.id" />
          <AppBadge
            :tone="project.status === 'ACTIVE' ? 'accent' : project.status === 'DONE' ? 'safe' : 'neutral'"
            dot
          >
            {{ PROJECT_STATUS_LABEL[project.status] }}
          </AppBadge>
        </template>
        <template #actions>
          <span class="mr-1 flex items-center gap-2 text-small" :class="store.saveState === 'unsaved' ? 'text-warn' : 'text-text-subtle'">
            <span
              class="size-1.5 rounded-full"
              :class="{
                'bg-safe': store.saveState === 'saved',
                'bg-warn': store.saveState === 'unsaved',
                'animate-pulse bg-accent': store.saveState === 'saving',
                settle: justSaved
              }"
              aria-hidden="true"
            />
            {{ store.saveState === 'saved' ? 'Saved' : store.saveState === 'saving' ? 'Saving' : 'Unsaved' }}
          </span>
        </template>
      </AppCatalogHeader>

      <div class="mx-auto flex min-h-[540px] w-full max-w-[1280px] flex-1 basis-0 flex-col px-6" data-testid="project-workspace">
        <SubNav class="mt-4 shrink-0" :model-value="tab" :items="tabItems" label="Project pages" />
        <div class="grid min-h-0 flex-1 grid-rows-[minmax(0,1fr)] py-4">
          <Transition name="fade-quick" mode="out-in">
            <ProjectTasksTab v-if="tab === 'tasks'" key="tasks" :project="project" @open-task="openTask" />
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

  </div>
</template>
