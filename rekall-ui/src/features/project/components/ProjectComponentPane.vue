<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import AppModeToggle from '@/components/ui/AppModeToggle.vue'
import FolderPickerButton from '@/features/project/components/FolderPickerButton.vue'
import ProjectComponentOutline from '@/features/project/components/ProjectComponentOutline.vue'
import ProjectPaneFrame from '@/features/project/components/ProjectPaneFrame.vue'
import ProjectReadingColumn from '@/features/project/components/ProjectReadingColumn.vue'
import TaskWidthMenu from '@/features/task/components/TaskWidthMenu.vue'
import { usePreferencesStore } from '@/features/settings/store/preferences.store'
import { relativeTime } from '@/common/format/relative-time'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useEditorMode } from '@/composables/useEditorMode'
import { relativeToRoot } from '@/features/project/lib/component-tree'
import { jumpToHeading } from '@/features/project/lib/heading-jump'
import { outlineOf } from '@/features/project/lib/markdown-outline'
import { useProjectComponentsStore } from '@/features/project/store/projectComponents.store'
import type { Direction } from '@/features/project/lib/section-order'
import type { ProjectComponent } from '@/features/project/model/project-component'

/**
 * One component as a page: what it is for, then the structured description a developer reads to
 * understand it and change it. Reading is the default; writing swaps the page for its fields and
 * the markdown source, and every field saves on its own after a pause in typing.
 */
const props = defineProps<{
  component: ProjectComponent
  repoFolder: string | null
  canMoveUp: boolean
  canMoveDown: boolean
}>()

const emit = defineEmits<{ move: [direction: Direction]; delete: [] }>()

const SAVE_DELAY_MS = 700
const SUMMARY_MAX_CHARACTERS = 500

const store = useProjectComponentsStore()
const preferences = usePreferencesStore()
const { run } = useAsyncAction()

const nameDraft = ref('')
const folderDraft = ref('')
const summaryDraft = ref('')
const descriptionDraft = ref('')
const { mode, openFor } = useEditorMode('read')
const saveState = ref<'saved' | 'unsaved' | 'saving'>('saved')
const article = ref<HTMLElement | null>(null)
let timer: ReturnType<typeof setTimeout> | null = null

const saveLabel = computed(() => ({ saved: 'Saved', unsaved: 'Unsaved', saving: 'Saving' })[saveState.value])
const headings = computed(() => outlineOf(descriptionDraft.value))
const hasDescription = computed(() => descriptionDraft.value.trim() !== '')
const shownFolder = computed(() => (props.repoFolder ? relativeToRoot(props.component.folder, props.repoFolder) : null) ?? props.component.folder)

function adopt(component: ProjectComponent): void {
  nameDraft.value = component.name
  folderDraft.value = component.folder
  summaryDraft.value = component.summary
  descriptionDraft.value = component.description
}

function schedule(): void {
  saveState.value = 'unsaved'
  if (timer) clearTimeout(timer)
  timer = setTimeout(() => void save(), SAVE_DELAY_MS)
}

async function save(): Promise<void> {
  if (timer) clearTimeout(timer)
  timer = null
  saveState.value = 'saving'
  const name = nameDraft.value.trim() === '' ? props.component.name : nameDraft.value.trim()
  const folder = folderDraft.value.trim() === '' ? props.component.folder : folderDraft.value.trim()
  const saved = await run(() =>
    store.update(props.component.id, { name, folder, summary: summaryDraft.value, description: descriptionDraft.value })
  )
  if (saved === null) {
    adopt(props.component)
    saveState.value = 'unsaved'
    return
  }
  if (timer === null) saveState.value = 'saved'
}

function folderChosen(chosen: string): void {
  folderDraft.value = chosen
  schedule()
}

// A component is its own pane instance, so it is read once; later changes from a session land only while nothing is being typed.
watch(
  () => props.component,
  (component) => {
    if (timer === null) adopt(component)
  },
  { immediate: true }
)

watch(
  () => props.component.id,
  () => {
    openFor(props.component.description)
  },
  { immediate: true }
)

onUnmounted(() => {
  if (timer) void save()
})
</script>

<template>
  <ProjectPaneFrame :label="component.name" testid="component-pane">
    <template #header>
      <div class="flex flex-wrap items-start gap-3 px-5 py-3">
        <div class="min-w-0 flex-1">
          <label class="sr-only" :for="`component-name-${component.id}`">Component name</label>
          <input
            :id="`component-name-${component.id}`"
            v-model="nameDraft"
            maxlength="200"
            autocomplete="off"
            spellcheck="false"
            class="focus-ring -mx-1 w-full truncate rounded-key border-0 bg-transparent px-1 py-0.5 text-heading font-medium tracking-[-0.01em] text-text hover:bg-surface-raised focus:bg-surface-raised"
            data-testid="component-name"
            @input="schedule"
            @keydown.enter.prevent="($event.target as HTMLInputElement).blur()"
          />
          <div v-if="mode === 'write'" class="mt-1.5 flex items-center gap-1.5">
            <label class="sr-only" :for="`component-folder-${component.id}`">Component folder</label>
            <input
              :id="`component-folder-${component.id}`"
              v-model="folderDraft"
              class="field text-text h-8 min-w-0 flex-1 rounded-control px-2 font-mono text-caption"
              maxlength="1000"
              autocomplete="off"
              spellcheck="false"
              placeholder="/path/to/the/component"
              data-testid="component-folder"
              @input="schedule"
            />
            <FolderPickerButton :root="repoFolder" :current="folderDraft" data-testid="component-browse" @pick="folderChosen" />
          </div>
          <p v-else class="mt-0.5 truncate font-mono text-caption text-text-muted" :title="component.folder" data-testid="component-folder-line">
            {{ shownFolder }}
          </p>
          <p class="mt-1 text-small text-text-subtle">
            Edited {{ relativeTime(component.updatedAt) }}
            <span class="px-1 text-border-strong" aria-hidden="true">/</span>
            <span :class="saveState === 'unsaved' ? 'text-warn' : ''" role="status" data-testid="component-save-state">{{ saveLabel }}</span>
          </p>
        </div>
        <div class="flex items-center gap-0.5">
          <AppButton
            variant="quiet"
            size="icon"
            class="disabled:opacity-30"
            :disabled="!canMoveUp"
            aria-label="Move component up"
            title="Move up"
            data-testid="component-up"
            @click="emit('move', 'up')"
          >
            <svg class="size-3.5" viewBox="0 0 14 14" fill="none" aria-hidden="true">
              <path d="M3.5 8.5L7 5l3.5 3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
          </AppButton>
          <AppButton
            variant="quiet"
            size="icon"
            class="disabled:opacity-30"
            :disabled="!canMoveDown"
            aria-label="Move component down"
            title="Move down"
            data-testid="component-down"
            @click="emit('move', 'down')"
          >
            <svg class="size-3.5" viewBox="0 0 14 14" fill="none" aria-hidden="true">
              <path d="M3.5 5.5L7 9l3.5-3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
          </AppButton>
          <AppButton variant="danger-quiet" size="sm" data-testid="component-delete" @click="emit('delete')">Delete</AppButton>
        </div>
        <TaskWidthMenu v-model="preferences.taskWidth" />
        <AppModeToggle v-model="mode" testid-prefix="component-mode" />
      </div>
    </template>

    <template #default="{ editorHeight }">
      <Transition name="fade-quick" mode="out-in">
        <ProjectReadingColumn v-if="mode === 'write'" key="write" class="flex flex-col gap-3">
          <div>
            <div class="mb-1 flex items-baseline justify-between gap-3">
              <label class="text-caption text-text-muted" :for="`component-summary-${component.id}`">Summary</label>
              <span
                class="font-mono text-micro tabular-nums"
                :class="summaryDraft.length > SUMMARY_MAX_CHARACTERS ? 'text-danger' : 'text-text-subtle'"
              >
                {{ summaryDraft.length }}/{{ SUMMARY_MAX_CHARACTERS }}
              </span>
            </div>
            <textarea
              :id="`component-summary-${component.id}`"
              v-model="summaryDraft"
              rows="3"
              class="field text-text block w-full resize-y rounded-control p-2 text-small leading-relaxed"
              placeholder="What it is for and what it owns, in two or three sentences. Every context that loads the project carries this."
              data-testid="component-summary"
              @input="schedule"
            />
          </div>
          <div>
            <p class="mb-1 text-caption text-text-muted">Description</p>
            <AppMarkdownEditor
              v-model="descriptionDraft"
              :height="editorHeight"
              grow
              :path-root="repoFolder"
              placeholder="Purpose, interfaces, configuration, how to run it and where a change goes. Tables and mermaid diagrams are drawn."
              @update:model-value="schedule"
            />
          </div>
        </ProjectReadingColumn>

        <div v-else key="read" class="@container">
          <div class="grid gap-x-10 @4xl:grid-cols-[minmax(0,1fr)_184px]">
            <article ref="article" class="min-w-0" data-testid="component-page">
              <ProjectReadingColumn>
                <p v-if="summaryDraft" class="mb-5 text-title font-normal leading-relaxed text-text" data-testid="component-summary-read">
                  {{ summaryDraft }}
                </p>
                <AppMarkdownEditor v-if="hasDescription" :model-value="descriptionDraft" readonly fit-column :read-settings="false" :path-root="repoFolder" />
                <div
                  v-else
                  class="max-w-[56ch] rounded-card border border-dashed border-border px-5 py-5 text-small leading-relaxed text-text-subtle"
                  data-testid="component-no-description"
                >
                  Nothing is written about this component beyond its summary. <span class="text-text-muted">Scan folder</span> has a
                  session read it and write the page, or switch to Write and describe it yourself.
                </div>
              </ProjectReadingColumn>
            </article>
            <ProjectComponentOutline
              v-if="headings.length > 1"
              class="hidden @4xl:flex"
              :headings="headings"
              @jump="jumpToHeading(article, $event)"
            />
          </div>
        </div>
      </Transition>
    </template>
  </ProjectPaneFrame>
</template>
