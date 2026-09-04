<script setup lang="ts">
import { computed, nextTick, onUnmounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import AppCatalogHeader from '@/components/catalog/AppCatalogHeader.vue'
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

// ------------------------------------------------------------------ description

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

// ------------------------------------------------------------------ the folder a session opens in

const folderDraft = ref('')
let folderTimer: ReturnType<typeof setTimeout> | null = null

/** Fixed for the lifetime of the page: the bridge is installed before the application boots. */
const canBrowse = desktopHost() !== null

function scheduleFolderSave(): void {
  if (folderTimer) clearTimeout(folderTimer)
  folderTimer = setTimeout(() => {
    if (project.value) void store.saveProjectRepoFolder(project.value.id, folderDraft.value)
  }, 700)
}

function onFolderInput(event: Event): void {
  folderDraft.value = (event.target as HTMLInputElement).value
  scheduleFolderSave()
}

/** A dismissed panel leaves what is typed alone, exactly as it does on the database field. */
async function browseFolder(): Promise<void> {
  const chosen = await pickFolder(folderDraft.value)
  if (!chosen) return
  folderDraft.value = chosen
  scheduleFolderSave()
}

// ------------------------------------------------------------------ blueprint

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
  <div class="min-h-full bg-canvas">
    <template v-if="!store.isLoading && !project">
      <AppCatalogHeader title="Project" />
      <div class="mx-auto max-w-[1240px] px-8 py-6">
        <AppEmptyState title="No project here" description="It may have been deleted, or the link is stale.">
          <RouterLink to="/projects"><AppButton variant="secondary" size="sm">Back to projects</AppButton></RouterLink>
        </AppEmptyState>
      </div>
    </template>

    <template v-else-if="!project">
      <div class="flex h-(--spacing-header) items-center border-b border-border px-8" aria-hidden="true">
        <div class="skeleton h-4 w-64" />
      </div>
      <div class="mx-auto flex max-w-[1240px] flex-col gap-4 px-8 py-6" aria-hidden="true">
        <div class="skeleton h-32 rounded-[var(--radius-card)]" />
        <div class="skeleton h-64 rounded-[var(--radius-card)]" />
      </div>
    </template>

    <template v-else>
      <AppCatalogHeader :title="company ? `${company.name} / ${project.title}` : project.title">
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
            data-testid="copy-project-anchor"
            @click="copyAnchor"
          >
            <span class="opacity-60">/rk</span>
            <span>{{ project.anchor }}</span>
            <span class="opacity-70">{{ copied ? 'copied' : 'copy' }}</span>
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
            class="focus-ring block w-full resize-none overflow-hidden rounded-[var(--radius-control)] border border-border bg-canvas p-3 text-[13.5px] leading-relaxed text-text outline-none transition-colors placeholder:text-text-subtle hover:border-border-strong focus:border-accent"
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
              class="focus-ring h-9 min-w-0 flex-1 rounded-[var(--radius-control)] border border-border bg-canvas px-3 font-mono text-[12.5px] text-text outline-none transition-colors placeholder:text-text-subtle hover:border-border-strong focus:border-accent"
              placeholder="/Users/you/Projects/thing"
              @input="onFolderInput"
            />
            <AppButton v-if="canBrowse" size="sm" variant="secondary" @click="browseFolder">
              Browse
            </AppButton>
          </div>
          <p class="mt-2 text-[12px] text-text-subtle">
            Where <span class="text-text-muted">Open in Claude Code</span> starts the session. Claude
            Code keeps the folder it was launched from for the whole session, so this is what decides
            which repository the work happens in.
          </p>
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

    <RecordDialog v-if="editing" :draft="editing" @close="editing = null" @saved="onDialogSaved" />
  </div>
</template>
