<script setup lang="ts">
import { computed, nextTick, onUnmounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import AppCatalogHeader from '@/components/catalog/AppCatalogHeader.vue'
import AutoCommitSwitch from '@/components/catalog/AutoCommitSwitch.vue'
import CopyGlyph from '@/components/ui/CopyGlyph.vue'
import RecordDialog from '@/components/console/RecordDialog.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import ProjectTrace from '@/components/ui/ProjectTrace.vue'
import { useConsoleStore } from '@/stores/console.store'
import { identityHue } from '@/common/identity'
import { hasActivity, projectActivitySeries } from '@/common/trace/activity-series'
import { projectDraft } from '@/model/record-draft'
import { PROJECT_STATUS_LABEL, TASK_STATUS_COLOR, TASK_STATUS_LABEL } from '@/model/catalog'
import { asProjectId } from '@/model/branded'
import { rkCommand } from '@/common/format/rk-command'
import { desktopHost, pickFolder } from '@/common/native/desktop'
import { fetchProjectRepository } from '@/api/catalog.api'
import type { ProjectRepository } from '@/model/catalog'
import LaunchClaudeCodeButton from '@/components/claude/LaunchClaudeCodeButton.vue'
import type { RecordDraft } from '@/model/record-draft'

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

const activitySeries = computed(() => {
  if (!project.value) return undefined
  const series = projectActivitySeries(project.value.id, store.tasks, store.timeEntries, Date.now())
  return hasActivity(series) ? series : undefined
})

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

const copied = ref(false)
async function copyAnchor(): Promise<void> {
  if (!project.value) return
  await navigator.clipboard?.writeText(rkCommand(project.value.anchor))
  copied.value = true
  setTimeout(() => (copied.value = false), 1400)
}

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

const blueprintDraft = ref('')
const blueprintMode = ref<'write' | 'read'>('read')
let blueprintTimer: ReturnType<typeof setTimeout> | null = null

function scheduleBlueprintSave(): void {
  if (blueprintTimer) clearTimeout(blueprintTimer)
  blueprintTimer = setTimeout(() => {
    if (project.value) void store.saveProjectBlueprint(project.value.id, blueprintDraft.value)
  }, 700)
}

watch(
  () => project.value?.id ?? null,
  async () => {
    if (descriptionTimer) clearTimeout(descriptionTimer)
    if (blueprintTimer) clearTimeout(blueprintTimer)
    if (folderTimer) clearTimeout(folderTimer)
    descriptionDraft.value = project.value?.description ?? ''
    folderDraft.value = project.value?.repoFolder ?? ''
    blueprintDraft.value = project.value?.blueprintMarkdown ?? ''
    blueprintMode.value = blueprintDraft.value.trim() ? 'read' : 'write'
    repository.value = null
    void readRepository()
    await nextTick()
    resize(descriptionArea.value)
  },
  { immediate: true }
)

onUnmounted(() => {
  if (descriptionTimer) {
    clearTimeout(descriptionTimer)
    if (project.value) void store.saveProjectDescription(project.value.id, descriptionDraft.value)
  }
  if (blueprintTimer) {
    clearTimeout(blueprintTimer)
    if (project.value) void store.saveProjectBlueprint(project.value.id, blueprintDraft.value)
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
      <div class="mx-auto max-w-[1240px] px-8 py-6">
        <AppEmptyState title="No project here" description="It may have been deleted, or the link is stale.">
          <RouterLink to="/projects"><AppButton variant="secondary" size="sm">Back to projects</AppButton></RouterLink>
        </AppEmptyState>
      </div>
    </template>

    <template v-else-if="!project">
      <Teleport to="#app-header-extra">
        <span class="h-5 w-px shrink-0 bg-border-strong" aria-hidden="true" />
        <div class="flex min-w-0 flex-1 items-center gap-3" aria-hidden="true">
          <div class="skeleton h-4 w-64" />
        </div>
      </Teleport>
      <div class="mx-auto flex max-w-[1240px] flex-col gap-4 px-8 py-6" aria-hidden="true">
        <div class="skeleton h-32 rounded-[var(--radius-card)]" />
        <div class="skeleton h-64 rounded-[var(--radius-card)]" />
      </div>
    </template>

    <template v-else>
      <AppCatalogHeader :title="project.title" :parent="company?.name">
        <template #title-suffix>
          <ProjectTrace :id="project.id" size="md" :series="activitySeries" />
          <AppBadge
            :tone="project.status === 'ACTIVE' ? 'accent' : project.status === 'DONE' ? 'safe' : 'neutral'"
            dot
          >
            {{ PROJECT_STATUS_LABEL[project.status] }}
          </AppBadge>
          <button
            class="anchor-chip focus-ring flex items-center gap-2 px-2.5 py-1 text-[11.5px] transition-colors hover:border-anchor"
            :class="copied && 'flash'"
            :title="copied ? 'Copied' : `Copy ${rkCommand(project.anchor)}`"
            data-testid="copy-project-anchor"
            @click="copyAnchor"
          >
            <span class="opacity-60">/rk</span>
            <span>{{ project.anchor }}</span>
            <CopyGlyph :copied="copied" />
          </button>
          <LaunchClaudeCodeButton
            :anchors="project.anchor"
            :folder="project.repoFolder"
            missing-hint="Set this project's folder below to open a session from it"
          />
        </template>
        <template #actions>
          <span class="mr-1 flex items-center gap-2 text-[12px]" :class="store.saveState === 'unsaved' ? 'text-warn' : 'text-text-subtle'">
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
          <AppButton variant="secondary" size="sm" data-testid="rename-project" @click="openRename">
            Rename / move
          </AppButton>
        </template>
      </AppCatalogHeader>

      <div class="mx-auto flex max-w-[1240px] flex-col gap-5 px-8 py-6">
        <AppCard :style="{ borderTop: `2px solid ${identityHue(project.id).line}` }">
          <p class="mb-2 eyebrow text-[11px]">
            Description
          </p>
          <textarea
            ref="descriptionArea"
            :value="descriptionDraft"
            data-testid="project-description"
            rows="2"
            class="field text-text block w-full resize-none overflow-hidden rounded-[var(--radius-control)] p-3 text-[13.5px] leading-relaxed"
            placeholder="What this is, in a few sentences. Travels into every context that loads this project."
            @input="onDescriptionInput"
          />
        </AppCard>

        <AppCard>
          <p class="mb-2 eyebrow text-[11px]">
            Folder
          </p>
          <div class="flex items-center gap-2">
            <input
              :value="folderDraft"
              data-testid="project-repo-folder"
              spellcheck="false"
              autocomplete="off"
              class="field text-text h-9 min-w-0 flex-1 rounded-[var(--radius-control)] px-3 font-mono text-[12.5px]"
              placeholder="/Users/you/Projects/thing"
              @input="onFolderInput"
            />
            <AppButton v-if="canBrowse" size="sm" variant="secondary" @click="browseFolder">
              Browse
            </AppButton>
          </div>
          <p class="mt-2 text-[12px] text-text-subtle">
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

        <AppCard :padded="false">
          <div class="relative overflow-hidden rounded-t-[var(--radius-card)]">
            <div
              class="texture-grid pointer-events-none absolute inset-0"
              :style="{ '--texture-tint': identityHue(project.id).base }"
              aria-hidden="true"
            />
            <div class="relative flex items-center gap-3 border-b border-border px-5 py-4">
              <div class="min-w-0 flex-1">
                <p class="text-[11px] font-semibold tracking-[0.02em] text-anchor">Blueprint</p>
                <p class="mt-0.5 truncate text-[12px] text-text-subtle">
                  What <code class="text-anchor/80">/rk {{ project.anchor }}</code> hands to Claude
                </p>
              </div>
              <div class="flex gap-0.5 rounded-[7px] bg-surface p-0.5">
                <button
                  v-for="option in (['write', 'read'] as const)"
                  :key="option"
                  class="focus-ring h-6 rounded-[5px] px-2.5 text-[11.5px] capitalize transition-colors"
                  :class="blueprintMode === option ? 'bg-surface-raised text-text' : 'text-text-subtle hover:text-text'"
                  :aria-pressed="blueprintMode === option"
                  @click="blueprintMode = option"
                >
                  {{ option }}
                </button>
              </div>
            </div>
          </div>
          <div class="p-4">
            <AppMarkdownEditor
              v-if="blueprintMode === 'write'"
              v-model="blueprintDraft"
              height="420px"
              :path-root="project.repoFolder"
              placeholder="# What this is

How it's built, how it's organised, the conventions to follow while working in it."
              @update:model-value="scheduleBlueprintSave"
            />
            <AppMarkdownEditor v-else :model-value="blueprintDraft" readonly />
          </div>
        </AppCard>

        <AppCard>
          <p class="mb-3 eyebrow text-[11px]">
            Tasks &middot; {{ projectTasks.length }}
          </p>
          <p v-if="!projectTasks.length" class="text-[12.5px] text-text-subtle">No task on this project yet.</p>
          <div v-else class="flex flex-col divide-y divide-border">
            <div
              v-for="task in projectTasks"
              :key="task.id"
              class="flex items-center gap-2.5 py-2 first:pt-0 last:pb-0"
            >
              <span class="size-[7px] shrink-0 rounded-full" :class="TASK_STATUS_COLOR[task.status]" aria-hidden="true" />
              <span class="min-w-0 flex-1">
                <span class="block truncate text-[13px] text-text">{{ task.title }}</span>
                <span class="block truncate font-mono text-[10.5px] text-anchor/80">{{ task.label }}</span>
              </span>
              <span class="shrink-0 text-[11px] text-text-subtle">{{ TASK_STATUS_LABEL[task.status] }}</span>
              <button
                class="focus-ring shrink-0 rounded-[var(--radius-control)] border border-border-strong px-2.5 py-1 text-[11.5px] text-text-muted transition-colors hover:border-accent hover:text-accent"
                @click="openInConsole(task.id)"
              >
                Open in console
              </button>
            </div>
          </div>
        </AppCard>
      </div>
    </template>

    <Transition name="dialog">
      <RecordDialog v-if="editing" :draft="editing" @close="editing = null" @saved="onDialogSaved" />
    </Transition>
  </div>
</template>
