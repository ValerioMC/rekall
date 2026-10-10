<script setup lang="ts">
import { computed, nextTick, onUnmounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import AppCatalogHeader from '@/components/catalog/AppCatalogHeader.vue'
import AutoCommitSwitch from '@/components/catalog/AutoCommitSwitch.vue'
import RecordDialog from '@/components/console/RecordDialog.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import ProjectBlueprintPane from '@/components/project/ProjectBlueprintPane.vue'
import ProjectComponentsSection from '@/components/project/ProjectComponentsSection.vue'
import ProjectDocsRail from '@/components/project/ProjectDocsRail.vue'
import ProjectSectionPane from '@/components/project/ProjectSectionPane.vue'
import ProjectTasksDrawer from '@/components/project/ProjectTasksDrawer.vue'
import ProjectMark from '@/components/ui/ProjectMark.vue'
import TokenCount from '@/components/ui/TokenCount.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useProjectSectionsStore } from '@/stores/projectSections.store'
import { useProjectSectionStream } from '@/composables/useProjectSectionStream'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { canMove, type Direction } from '@/common/project/section-order'
import { identityHue } from '@/common/identity'
import { useTokenUsageStore } from '@/stores/tokenUsage.store'
import { projectDraft } from '@/model/record-draft'
import { PROJECT_STATUS_LABEL } from '@/model/catalog'
import { asProjectId } from '@/model/branded'
import { desktopHost, pickFolder } from '@/common/native/desktop'
import { fetchProjectRepository } from '@/api/catalog.api'
import type { ProjectRepository } from '@/model/catalog'
import type { RecordDraft } from '@/model/record-draft'
import type { RailSelection } from '@/components/project/ProjectDocsRail.vue'
import type { ProjectSection } from '@/model/project-section'

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

const descriptionDraft = ref('')
const descriptionArea = ref<HTMLTextAreaElement | null>(null)
let descriptionTimer: ReturnType<typeof setTimeout> | null = null

function resize(el: HTMLTextAreaElement | null): void {
  if (!el) return
  el.style.height = 'auto'
  el.style.height = `${el.scrollHeight}px`
}

function onDescriptionInput(event: Event): void {
  descriptionDraft.value = (event.target as HTMLTextAreaElement).value
  resize(event.target as HTMLTextAreaElement)
  if (descriptionTimer) clearTimeout(descriptionTimer)
  descriptionTimer = setTimeout(() => {
    if (project.value) void store.saveProjectDescription(project.value.id, descriptionDraft.value)
  }, 700)
}

const folderDraft = ref('')
let folderTimer: ReturnType<typeof setTimeout> | null = null

const canBrowse = desktopHost() !== null

// What git makes of the folder: read once per project and again after every folder save,
// since the switch below is only offered on a repository.
const repository = ref<ProjectRepository | null>(null)
const repositoryPending = ref(false)
const autoCommitSaving = ref(false)

async function readRepository(): Promise<void> {
  const id = project.value?.id
  if (!id) return
  repositoryPending.value = true
  try {
    const status = await fetchProjectRepository(id)
    if (project.value?.id === id) repository.value = status
  } catch {
    if (project.value?.id === id) repository.value = null
  } finally {
    if (project.value?.id === id) repositoryPending.value = false
  }
}

async function saveFolder(): Promise<void> {
  if (!project.value) return
  await store.saveProjectRepoFolder(project.value.id, folderDraft.value)
  await readRepository()
}

function scheduleFolderSave(): void {
  if (folderTimer) clearTimeout(folderTimer)
  folderTimer = setTimeout(() => void saveFolder(), 700)
}

async function setAutoCommit(enabled: boolean): Promise<void> {
  if (!project.value) return
  autoCommitSaving.value = true
  try {
    await store.saveProjectAutoCommit(project.value.id, enabled)
  } finally {
    autoCommitSaving.value = false
  }
}

function onFolderInput(event: Event): void {
  folderDraft.value = (event.target as HTMLInputElement).value
  scheduleFolderSave()
}

async function browseFolder(): Promise<void> {
  const chosen = await pickFolder(folderDraft.value)
  if (!chosen) return
  folderDraft.value = chosen
  scheduleFolderSave()
}

watch(
  () => project.value?.id ?? null,
  async () => {
    if (descriptionTimer) clearTimeout(descriptionTimer)
    if (folderTimer) clearTimeout(folderTimer)
    descriptionDraft.value = project.value?.description ?? ''
    folderDraft.value = project.value?.repoFolder ?? ''
    selection.value = 'blueprint'
    showingTasks.value = false
    if (project.value) void run(() => sectionsStore.load(project.value!.id))
    repository.value = null
    void readRepository()
    await nextTick()
    resize(descriptionArea.value)
  },
  { immediate: true }
)

watch(sections, (list) => {
  if (selection.value !== 'blueprint' && !list.some((section) => section.id === selection.value)) selection.value = 'blueprint'
})

onUnmounted(() => {
  if (descriptionTimer) {
    clearTimeout(descriptionTimer)
    if (project.value) void store.saveProjectDescription(project.value.id, descriptionDraft.value)
  }
  if (folderTimer) {
    clearTimeout(folderTimer)
    if (project.value) void store.saveProjectRepoFolder(project.value.id, folderDraft.value)
  }
})
</script>

<template>
  <div class="flex-1 min-h-0 overflow-y-auto bg-canvas">
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

      <div class="mx-auto flex w-full max-w-[1800px] flex-col gap-5 px-6 py-6 2xl:px-10">
        <div class="grid gap-5 lg:grid-cols-2">
          <AppCard :style="{ borderTop: `2px solid ${identityHue(project.id).line}` }">
            <p class="mb-2 eyebrow text-caption">
              Description
            </p>
            <textarea
              ref="descriptionArea"
              :value="descriptionDraft"
              data-testid="project-description"
              rows="3"
              class="field text-text block w-full resize-none overflow-hidden rounded-control p-3 text-body leading-relaxed"
              placeholder="What this is, in a few sentences. Travels into every context that loads this project."
              @input="onDescriptionInput"
            />
          </AppCard>

          <AppCard>
            <p class="mb-2 eyebrow text-caption">
              Folder
            </p>
            <div class="flex items-center gap-2">
              <input
                :value="folderDraft"
                data-testid="project-repo-folder"
                spellcheck="false"
                autocomplete="off"
                class="field text-text h-9 min-w-0 flex-1 rounded-control px-3 font-mono text-small"
                placeholder="/Users/you/Projects/thing"
                @input="onFolderInput"
              />
              <AppButton v-if="canBrowse" size="sm" variant="secondary" @click="browseFolder">
                Browse
              </AppButton>
            </div>
            <p class="mt-2 text-small text-text-subtle">
              Where <span class="text-text-muted">Open in terminal</span> starts the session. Claude
              Code keeps the folder it was launched from for the whole session, so this is what decides
              which repository the work happens in.
            </p>
            <div class="mt-4">
              <AutoCommitSwitch
                :model-value="project.autoCommit"
                :repository="repository"
                :pending="repositoryPending"
                :saving="autoCommitSaving"
                @update:model-value="setAutoCommit"
              />
            </div>
          </AppCard>
        </div>

        <ProjectComponentsSection :project-id="project.id" :project-title="project.title" :repo-folder="project.repoFolder" />

        <div class="grid items-start gap-5 lg:grid-cols-[264px_minmax(0,1fr)]">
          <ProjectDocsRail
            class="lg:sticky lg:top-4"
            :sections="sections"
            :selected="selection"
            :loading="sectionsStore.isLoading(project.id)"
            :creating="creatingSection"
            @select="selection = $event"
            @create="createSection"
          />
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
