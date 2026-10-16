<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import AppCatalogHeader from '@/features/catalog/components/AppCatalogHeader.vue'
import RecordDialog from '@/features/catalog/components/RecordDialog.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import ProjectAboutGroup from '@/features/project/components/ProjectAboutGroup.vue'
import ProjectBlueprintPane from '@/features/project/components/ProjectBlueprintPane.vue'
import ProjectComponentsSection from '@/features/project/components/ProjectComponentsSection.vue'
import ProjectDocsRail from '@/features/project/components/ProjectDocsRail.vue'
import ProjectRepositoryGroup from '@/features/project/components/ProjectRepositoryGroup.vue'
import ProjectSectionPane from '@/features/project/components/ProjectSectionPane.vue'
import ProjectTasksDrawer from '@/features/project/components/ProjectTasksDrawer.vue'
import ProjectMark from '@/components/ui/ProjectMark.vue'
import TokenCount from '@/components/ui/TokenCount.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useProjectSectionsStore } from '@/features/project/store/projectSections.store'
import { useProjectSectionStream } from '@/features/project/composables/useProjectSectionStream'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { canMove, type Direction } from '@/features/project/lib/section-order'
import { identityHue } from '@/common/identity'
import { useTokenUsageStore } from '@/features/tokens/store/tokenUsage.store'
import { projectDraft } from '@/features/catalog/model/record-draft'
import { PROJECT_STATUS_LABEL } from '@/features/catalog/model/catalog'
import { asProjectId } from '@/model/branded'
import type { RecordDraft } from '@/features/catalog/model/record-draft'
import type { RailSelection } from '@/features/project/components/ProjectDocsRail.vue'
import type { ProjectSection } from '@/features/project/model/project-section'

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
const { run } = useAsyncAction()

const sections = computed(() => sectionsStore.sectionsOf(projectId.value))
const selection = ref<RailSelection>('blueprint')
const selectedSection = computed<ProjectSection | null>(() =>
  selection.value === 'blueprint' ? null : (sections.value.find((section) => section.id === selection.value) ?? null)
)
const sectionIds = computed(() => sections.value.map((section) => section.id))
const showingTasks = ref(false)
const creatingSection = ref(false)
const deletingSection = ref<ProjectSection | null>(null)

useProjectSectionStream((event) => void sectionsStore.applyEvent(event.projectId))

async function createSection(title: string): Promise<void> {
  creatingSection.value = true
  const created = await run(() => sectionsStore.create(projectId.value, title))
  creatingSection.value = false
  if (created) selection.value = created.id
}

async function moveSection(direction: Direction): Promise<void> {
  if (selectedSection.value) await run(() => sectionsStore.move(projectId.value, selectedSection.value!.id, direction))
}

async function confirmDeleteSection(): Promise<void> {
  const target = deletingSection.value
  deletingSection.value = null
  if (!target) return
  const removed = await run(() => sectionsStore.remove(projectId.value, target.id).then(() => true), `Deleted ${target.title}`)
  if (removed && selection.value === target.id) selection.value = 'blueprint'
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
    selection.value = 'blueprint'
    showingTasks.value = false
    if (project.value) void run(() => sectionsStore.load(project.value!.id))
  },
  { immediate: true }
)

watch(sections, (list) => {
  if (selection.value !== 'blueprint' && !list.some((section) => section.id === selection.value)) selection.value = 'blueprint'
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

      <!-- Three areas, each about one thing: the pages, the page open, and the project's settings.
           From lg up the workspace fills the window and each area scrolls on its own, so the
           blueprint is read without scrolling the page. -->
      <div class="workspace mx-auto w-full max-w-[1800px] px-6 py-5 2xl:px-10" data-testid="project-workspace">
        <ProjectDocsRail
          class="workspace-nav"
          :sections="sections"
          :selected="selection"
          :loading="sectionsStore.isLoading(project.id)"
          :creating="creatingSection"
          @select="selection = $event"
          @create="createSection"
        />

        <div class="workspace-doc">
          <Transition name="fade-quick" mode="out-in">
            <ProjectSectionPane
              v-if="selectedSection"
              :key="selectedSection.id"
              :section="selectedSection"
              :repo-folder="project.repoFolder"
              :can-move-up="canMove(sectionIds, selectedSection.id, 'up')"
              :can-move-down="canMove(sectionIds, selectedSection.id, 'down')"
              @move="moveSection"
              @delete="deletingSection = selectedSection"
            />
            <ProjectBlueprintPane v-else key="blueprint" :project="project" />
          </Transition>
        </div>

        <aside
          class="workspace-inspect divide-y divide-border overflow-y-auto rounded-card border border-border bg-surface shadow-[var(--shadow-raised)]"
          :style="{ borderTop: `2px solid ${identityHue(project.id).line}` }"
          aria-label="Project settings"
          data-testid="project-inspector"
        >
          <ProjectAboutGroup :project="project" />
          <ProjectComponentsSection :project-id="project.id" :project-title="project.title" :repo-folder="project.repoFolder" />
          <ProjectRepositoryGroup :project="project" />
        </aside>
      </div>
    </template>

    <Transition name="slide-over">
      <ProjectTasksDrawer v-if="showingTasks" :tasks="projectTasks" @close="showingTasks = false" @open="openInConsole" />
    </Transition>

    <Transition name="dialog">
      <AppConfirm
        v-if="deletingSection"
        :title="`Delete ${deletingSection.title}?`"
        body="The section and everything written in it is removed. No history keeps it."
        :blast="`Deletes the section \u201c${deletingSection.title}\u201d from ${project?.title ?? 'this project'}.`"
        confirm-label="Delete section"
        @cancel="deletingSection = null"
        @confirm="confirmDeleteSection"
      />
    </Transition>

    <Transition name="dialog">
      <RecordDialog v-if="editing" :draft="editing" @close="editing = null" @saved="onDialogSaved" />
    </Transition>
  </div>
</template>

<style scoped>
.workspace {
  display: grid;
  gap: 16px;
  grid-template-areas: 'nav' 'doc' 'inspect';
}

.workspace-nav {
  grid-area: nav;
}

.workspace-doc {
  grid-area: doc;
  height: calc(100dvh - var(--spacing-header) - 40px);
  min-height: 480px;
}

.workspace-inspect {
  grid-area: inspect;
  min-height: 0;
}

/* From lg the workspace stops scrolling as a page: it takes the window, and the pages and the
   settings share the left column under one another. */
@media (width >= 64rem) {
  .workspace {
    flex: 1 1 0;
    min-height: 0;
    grid-template-columns: 248px minmax(0, 1fr);
    grid-template-rows: auto minmax(0, 1fr);
    grid-template-areas:
      'nav doc'
      'inspect doc';
  }

  .workspace-nav {
    max-height: 40vh;
    overflow-y: auto;
  }

  .workspace-doc {
    height: auto;
    min-height: 0;
  }
}

/* Wide enough for the settings to get a column of their own beside the page. */
@media (width >= 80rem) {
  .workspace {
    grid-template-columns: 232px minmax(0, 1fr) 340px;
    grid-template-rows: minmax(0, 1fr);
    grid-template-areas: 'nav doc inspect';
  }

  .workspace-nav {
    max-height: none;
  }
}
</style>
