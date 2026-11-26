<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import AppModeToggle from '@/components/ui/AppModeToggle.vue'
import ExportAction from '@/features/export/components/ExportAction.vue'
import ProjectReadingColumn from '@/features/project/components/ProjectReadingColumn.vue'
import TaskWidthMenu from '@/features/task/components/TaskWidthMenu.vue'
import { usePreferencesStore } from '@/features/settings/store/preferences.store'
import ProjectComponentOutline from '@/features/project/components/ProjectComponentOutline.vue'
import ProjectSectionContextChip from '@/features/project/components/ProjectSectionContextChip.vue'
import ProjectPaneFrame from '@/features/project/components/ProjectPaneFrame.vue'
import ProjectSectionGauge from '@/features/project/components/ProjectSectionGauge.vue'
import { relativeTime } from '@/common/format/relative-time'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useEditorMode } from '@/composables/useEditorMode'
import { useProjectSectionsStore } from '@/features/project/store/projectSections.store'
import { jumpToHeading } from '@/features/project/lib/heading-jump'
import { outlineOf } from '@/features/project/lib/markdown-outline'
import type { ExportSelection } from '@/features/export/model/project-export'
import type { ReferencePage } from '@/common/markdown/page-reference'
import type { Direction } from '@/features/project/lib/section-order'
import type { ProjectSection } from '@/features/project/model/project-section'
import type { SectionContext } from '@/features/project/model/section-context'
import type { SectionContextId } from '@/model/branded'

const SAVE_DELAY_MS = 700

const props = defineProps<{
  section: ProjectSection
  contexts: readonly SectionContext[]
  sections: readonly ProjectSection[]
  pages: readonly ReferencePage[]
  repoFolder: string | null
  canMoveUp: boolean
  canMoveDown: boolean
}>()

const emit = defineEmits<{
  move: [direction: Direction]
  moveToContext: [id: SectionContextId]
  createContext: [title: string]
  openPage: [title: string]
  delete: []
  replace: []
}>()

const store = useProjectSectionsStore()
const preferences = usePreferencesStore()
const { run } = useAsyncAction()

const titleDraft = ref('')
const bodyDraft = ref('')
const { mode, openFor } = useEditorMode('read')
const saveState = ref<'saved' | 'unsaved' | 'saving'>('saved')
const article = ref<HTMLElement | null>(null)
let timer: ReturnType<typeof setTimeout> | null = null

const headings = computed(() => outlineOf(bodyDraft.value))
const saveLabel = computed(() => ({ saved: 'Saved', unsaved: 'Unsaved', saving: 'Saving' })[saveState.value])

function adopt(section: ProjectSection): void {
  titleDraft.value = section.title
  bodyDraft.value = section.bodyMarkdown
}

function schedule(): void {
  saveState.value = 'unsaved'
  if (timer) clearTimeout(timer)
  timer = setTimeout(() => void save(), SAVE_DELAY_MS)
}

let writing: Promise<void> | null = null

function save(): Promise<void> {
  const started = write().finally(() => (writing = null))
  writing = started
  return started
}

async function settle(): Promise<void> {
  if (timer) await save()
  else await writing
}

const thisPage = computed<ExportSelection>(() => ({ blueprint: false, sectionIds: [props.section.id], componentIds: [] }))

async function write(): Promise<void> {
  if (timer) clearTimeout(timer)
  timer = null
  saveState.value = 'saving'
  const title = titleDraft.value.trim() === '' ? props.section.title : titleDraft.value
  const saved = await run(() => store.update(props.section.id, { title, bodyMarkdown: bodyDraft.value }))
  if (saved === null) {
    // Refused (a title another section has): show what is stored rather than what was refused.
    titleDraft.value = props.section.title
    saveState.value = 'unsaved'
    return
  }
  if (timer === null) saveState.value = 'saved'
}

// A section is its own pane instance, so it is read once; later changes from a session land only while nothing is being typed.
watch(
  () => props.section,
  (section) => {
    if (timer === null) adopt(section)
  },
  { immediate: true }
)

watch(
  () => props.section.id,
  () => {
    openFor(props.section.bodyMarkdown)
  },
  { immediate: true }
)

onUnmounted(() => {
  if (timer) void save()
})
</script>

<template>
  <ProjectPaneFrame :label="section.title" testid="section-pane">
    <template #header>
      <div class="px-5 py-3">
        <div class="flex items-center gap-3">
          <div class="min-w-0 flex-1">
            <label class="sr-only" :for="`section-title-${section.id}`">Section title</label>
            <input
              :id="`section-title-${section.id}`"
              v-model="titleDraft"
              maxlength="200"
              autocomplete="off"
              class="focus-ring w-full truncate rounded-key border-0 bg-transparent px-1 py-0.5 -mx-1 text-title font-semibold tracking-[-0.01em] text-text hover:bg-surface-raised focus:bg-surface-raised"
              data-testid="section-title"
              @input="schedule"
              @keydown.enter.prevent="($event.target as HTMLInputElement).blur()"
            />
          </div>
          <div class="flex shrink-0 items-center gap-2">
            <ExportAction :project-id="section.projectId" :selection="thisPage" :settle="settle" />
            <TaskWidthMenu v-model="preferences.taskWidth" />
            <AppModeToggle v-model="mode" testid-prefix="section-mode" />
          </div>
        </div>
        <div class="mt-0.5 flex flex-wrap items-center justify-between gap-x-3 gap-y-1">
          <p class="text-small text-text-subtle">
            Edited {{ relativeTime(section.updatedAt) }}
            <span class="px-1 text-border-strong" aria-hidden="true">/</span>
            <span :class="saveState === 'unsaved' ? 'text-warn' : ''" role="status" data-testid="section-save-state">{{ saveLabel }}</span>
          </p>
          <div class="flex flex-wrap items-center gap-1">
            <ProjectSectionContextChip
              :contexts="contexts"
              :sections="sections"
              :current-id="section.contextId"
              @move="emit('moveToContext', $event)"
              @create-context="emit('createContext', $event)"
            />
            <AppButton variant="quiet" size="sm" title="Replace the text with a markdown file" data-testid="section-replace" @click="emit('replace')">
              <svg class="size-3.5" viewBox="0 0 14 14" fill="none" aria-hidden="true">
                <path d="M7 9V2M4.5 4.3 7 1.8l2.5 2.5M2 8.8v1.7a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1V8.8" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
              Replace from file
            </AppButton>
            <AppButton
              variant="quiet"
              size="icon"
              class="disabled:opacity-30"
              :disabled="!canMoveUp"
              aria-label="Move page up in its context"
              title="Move up"
              data-testid="section-move-up"
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
              aria-label="Move page down in its context"
              title="Move down"
              data-testid="section-move-down"
              @click="emit('move', 'down')"
            >
              <svg class="size-3.5" viewBox="0 0 14 14" fill="none" aria-hidden="true">
                <path d="M3.5 5.5L7 9l3.5-3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
            </AppButton>
            <AppButton variant="danger-quiet" size="sm" data-testid="section-delete" @click="emit('delete')">Delete</AppButton>
          </div>
        </div>
      </div>
      <ProjectSectionGauge :characters="bodyDraft.length" />
    </template>
    <template #default="{ editorHeight }">
      <Transition name="fade-quick" mode="out-in">
        <ProjectReadingColumn v-if="mode === 'write'" key="write">
          <AppMarkdownEditor
            v-model="bodyDraft"
            :height="editorHeight"
            grow
            :path-root="repoFolder"
            :picture-owner="{ kind: 'PAGE', id: section.id }"
            :pages="pages"
            placeholder="Write in markdown. Anything about this project that isn't the blueprint."
            @update:model-value="schedule"
          />
        </ProjectReadingColumn>
        <div v-else key="read" class="@container">
          <div class="grid gap-x-10 @4xl:grid-cols-[minmax(0,1fr)_184px]">
            <article ref="article" class="min-w-0" data-testid="section-page">
              <AppMarkdownEditor
                :model-value="bodyDraft"
                readonly
                :read-settings="false"
                :path-root="repoFolder"
                :picture-owner="{ kind: 'PAGE', id: section.id }"
                :pages="pages"
                @open-page="emit('openPage', $event)"
              />
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
