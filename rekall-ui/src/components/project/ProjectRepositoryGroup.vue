<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue'
import AutoCommitSwitch from '@/components/catalog/AutoCommitSwitch.vue'
import InspectorGroup from '@/components/project/InspectorGroup.vue'
import AppButton from '@/components/ui/AppButton.vue'
import { fetchProjectRepository } from '@/api/catalog.api'
import { desktopHost, pickFolder } from '@/common/native/desktop'
import { useConsoleStore } from '@/stores/console.store'
import type { Project, ProjectRepository } from '@/model/catalog'
import type { ProjectId } from '@/model/branded'

/**
 * Where the project's sessions run: its folder, and whether a claim commits it. Git is asked
 * about the folder once per project and again after every folder save, since auto-commit is only
 * offered on a repository.
 */
const props = defineProps<{ project: Project }>()

const SAVE_DELAY_MS = 700

const store = useConsoleStore()
const canBrowse = desktopHost() !== null

const draft = ref('')
const repository = ref<ProjectRepository | null>(null)
const repositoryPending = ref(false)
const autoCommitSaving = ref(false)
let timer: ReturnType<typeof setTimeout> | null = null

const summary = computed(() => {
  const folder = props.project.repoFolder ?? 'No folder yet'
  return props.project.autoCommit ? `${folder} · commits on claim` : folder
})

async function readRepository(id: ProjectId): Promise<void> {
  repositoryPending.value = true
  try {
    const status = await fetchProjectRepository(id)
    if (props.project.id === id) repository.value = status
  } catch {
    if (props.project.id === id) repository.value = null
  } finally {
    if (props.project.id === id) repositoryPending.value = false
  }
}

async function save(id: ProjectId): Promise<void> {
  if (timer) clearTimeout(timer)
  timer = null
  await store.saveProjectRepoFolder(id, draft.value)
  if (props.project.id === id) await readRepository(id)
}

function schedule(): void {
  if (timer) clearTimeout(timer)
  const id = props.project.id
  timer = setTimeout(() => void save(id), SAVE_DELAY_MS)
}

function onInput(event: Event): void {
  draft.value = (event.target as HTMLInputElement).value
  schedule()
}

async function browse(): Promise<void> {
  const chosen = await pickFolder(draft.value)
  if (!chosen) return
  draft.value = chosen
  schedule()
}

async function setAutoCommit(enabled: boolean): Promise<void> {
  autoCommitSaving.value = true
  try {
    await store.saveProjectAutoCommit(props.project.id, enabled)
  } finally {
    autoCommitSaving.value = false
  }
}

watch(
  () => props.project.id,
  (id, previous) => {
    if (timer && previous) void save(previous)
    draft.value = props.project.repoFolder ?? ''
    repository.value = null
    void readRepository(id)
  },
  { immediate: true }
)

onUnmounted(() => {
  if (timer) void save(props.project.id)
})
</script>

<template>
  <InspectorGroup group="repository" title="Repository" :summary="summary">
    <label class="mb-1 block text-caption text-text-muted" for="project-repo-folder">Folder</label>
    <div class="flex items-center gap-2">
      <input
        id="project-repo-folder"
        :value="draft"
        data-testid="project-repo-folder"
        spellcheck="false"
        autocomplete="off"
        class="field text-text h-8 min-w-0 flex-1 rounded-control px-2.5 font-mono text-caption"
        placeholder="/Users/you/Projects/thing"
        @input="onInput"
      />
      <AppButton v-if="canBrowse" size="sm" variant="secondary" @click="browse">Browse</AppButton>
    </div>
    <p class="mt-1.5 text-caption leading-relaxed text-text-subtle">
      Where <span class="text-text-muted">Open in terminal</span> starts the session, so it decides which repository
      the work happens in.
    </p>
    <div class="mt-3">
      <AutoCommitSwitch
        :model-value="project.autoCommit"
        :repository="repository"
        :pending="repositoryPending"
        :saving="autoCommitSaving"
        @update:model-value="setAutoCommit"
      />
    </div>
  </InspectorGroup>
</template>
