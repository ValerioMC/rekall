<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import AppCatalogHeader from '@/features/catalog/components/AppCatalogHeader.vue'
import RecordDialog from '@/features/catalog/components/RecordDialog.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import ProjectBlueprintPane from '@/features/project/components/ProjectBlueprintPane.vue'
import ProjectComponentsPage from '@/features/project/components/ProjectComponentsPage.vue'
import ProjectPageTabs from '@/features/project/components/ProjectPageTabs.vue'
import ProjectSectionsPage from '@/features/project/components/ProjectSectionsPage.vue'
import ProjectSettingsPage from '@/features/project/components/ProjectSettingsPage.vue'
import ProjectConnectionsPage from '@/features/connection/components/ProjectConnectionsPage.vue'
import ProjectTasksDrawer from '@/features/project/components/ProjectTasksDrawer.vue'
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
import { projectDraft } from '@/features/catalog/model/record-draft'
import { PROJECT_STATUS_LABEL } from '@/features/catalog/model/catalog'
import { asProjectId } from '@/model/branded'
import type { RecordDraft } from '@/features/catalog/model/record-draft'
import { BLUEPRINT, tabOf, type ProjectPage, type ProjectTab } from '@/features/project/model/project-page'
import type { DeploymentGraphId, ProjectSectionId } from '@/model/branded'

const props = defineProps<{ id: string }>()

const store = useConsoleStore()
const router = useRouter()

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
const page = ref<ProjectPage>(BLUEPRINT)
const showingTasks = ref(false)

const counts = computed<Partial<Record<ProjectTab, number>>>(() => ({
  sections: sections.value.length,
  graphs: graphs.value.length,
  components: componentsStore.componentsOf(projectId.value).length,
  connections: bindings.value.length
}))
const warnings = computed<Partial<Record<ProjectTab, string>>>(() => {
  const missing = bindings.value.filter((binding) => !binding.credentialSetAt).length
  return missing ? { connections: missing === 1 ? '1 binding without a credential' : `${missing} bindings without a credential` } : {}
})

useProjectSectionStream((event) => void sectionsStore.applyEvent(event.projectId))
useConnectionStream((event) => {
  if (project.value && event.companyId === project.value.companyId) void loadConnections()
})

async function loadConnections(): Promise<void> {
  const shown = project.value
  if (!shown) return
  await run(() => Promise.all([connectionsStore.loadProject(shown.id), connectionsStore.load(shown.companyId)]))
}

/** A tab opens its first section or graph; pages with nothing to pick open as they are. */
function openTab(tab: ProjectTab): void {
  if (tab === tabOf(page.value)) return
  if (tab === 'sections') page.value = { kind: 'section', id: sections.value[0]?.id ?? null }
  else if (tab === 'graphs') page.value = { kind: 'graph', id: graphs.value[0]?.id ?? null }
  else page.value = { kind: tab }
}

function openSection(id: ProjectSectionId | null): void {
  page.value = { kind: 'section', id }
}

function openGraph(id: DeploymentGraphId | null): void {
  page.value = { kind: 'graph', id }
}

const tokenUsage = useTokenUsageStore()
const tokens = computed(() => tokenUsage.ofProject(projectId.value, store.tasks))

const editing = ref<RecordDraft | null>(null)

function openRename(): void {
  if (project.value) editing.value = projectDraft(project.value.companyId, project.value)
}

function onDialogSaved(): void {
  if (!store.projects.some((candidate) => candidate.id === projectId.value)) {
    router.push('/projects')
  }
}

function openInConsole(taskId: (typeof projectTasks.value)[number]['id']): void {
  if (!project.value) return
  showingTasks.value = false
  store.setScope(project.value.companyId, project.value.id)
  store.selectTask(taskId)
  router.push('/')
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
    page.value = BLUEPRINT
    showingTasks.value = false
    if (!project.value) return
    const id = project.value.id
    void run(() => sectionsStore.load(id))
    void run(() => graphsStore.load(id))
    void run(() => componentsStore.load(id))
    void loadConnections()
  },
  { immediate: true }
)

watch([sections, graphs], () => {
  const open = page.value
  if (open.kind === 'section' && open.id && !sections.value.some((section) => section.id === open.id)) openSection(null)
  if (open.kind === 'graph' && open.id && !graphs.value.some((graph) => graph.id === open.id)) openGraph(null)
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
          <AppButton variant="secondary" size="sm" data-testid="show-tasks" @click="showingTasks = true">
            Tasks
            <span class="font-mono text-caption text-text-subtle">{{ projectTasks.length }}</span>
          </AppButton>
          <AppButton variant="secondary" size="sm" data-testid="rename-project" @click="openRename">
            Rename / move
          </AppButton>
        </template>
      </AppCatalogHeader>

      <div
        class="mx-auto flex min-h-[540px] w-full max-w-[1800px] flex-1 basis-0 flex-col px-6 2xl:px-10"
        data-testid="project-workspace"
      >
        <ProjectPageTabs
          class="shrink-0 border-b border-border"
          :selected="tabOf(page)"
          :counts="counts"
          :warnings="warnings"
          @select="openTab"
        />
        <div class="grid min-h-0 flex-1 grid-rows-[minmax(0,1fr)] py-4">
          <Transition name="fade-quick" mode="out-in">
            <ProjectSectionsPage
              v-if="page.kind === 'section'"
              key="sections"
              :project="project"
              :selected-id="page.id"
              @open="openSection"
            />
            <ProjectGraphsPage
              v-else-if="page.kind === 'graph' || page.kind === 'graph-setup'"
              key="graphs"
              :project="project"
              :selected-id="page.kind === 'graph' ? page.id : null"
              :setting-up="page.kind === 'graph-setup'"
              @open="openGraph"
              @setup="page = { kind: 'graph-setup' }"
            />
            <ProjectComponentsPage
              v-else-if="page.kind === 'components'"
              key="components"
              :project-id="project.id"
              :project-title="project.title"
              :repo-folder="project.repoFolder"
            />
            <ProjectConnectionsPage v-else-if="page.kind === 'connections'" key="connections" :project="project" />
            <ProjectSettingsPage v-else-if="page.kind === 'settings'" key="settings" :project="project" />
            <ProjectBlueprintPane v-else key="blueprint" :project="project" />
          </Transition>
        </div>
      </div>
    </template>

    <Transition name="slide-over">
      <ProjectTasksDrawer v-if="showingTasks" :tasks="projectTasks" @close="showingTasks = false" @open="openInConsole" />
    </Transition>

    <Transition name="dialog">
      <RecordDialog v-if="editing" :draft="editing" @close="editing = null" @saved="onDialogSaved" />
    </Transition>
  </div>
</template>
