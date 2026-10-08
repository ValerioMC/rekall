<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import AppModeToggle from '@/components/ui/AppModeToggle.vue'
import { relativeTime } from '@/common/format/relative-time'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useViewportEditorHeight } from '@/composables/useViewportEditorHeight'
import { useProjectSectionsStore } from '@/stores/projectSections.store'
import type { Direction } from '@/common/project/section-order'
import type { ProjectSection } from '@/model/project-section'

const SAVE_DELAY_MS = 700
// Page header, pane header and card padding around the editor; the editor takes the rest of the window.
const PAGE_CHROME_PX = 230
const EDITOR_FLOOR_PX = 360

const props = defineProps<{
  section: ProjectSection
  repoFolder: string | null
  canMoveUp: boolean
  canMoveDown: boolean
}>()

const emit = defineEmits<{ move: [direction: Direction]; delete: [] }>()

const store = useProjectSectionsStore()
const { run } = useAsyncAction()
const editorMinimum = useViewportEditorHeight(PAGE_CHROME_PX, EDITOR_FLOOR_PX)

const titleDraft = ref('')
const bodyDraft = ref('')
const mode = ref<'write' | 'read'>('read')
const saveState = ref<'saved' | 'unsaved' | 'saving'>('saved')
let timer: ReturnType<typeof setTimeout> | null = null

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

async function save(): Promise<void> {
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
    mode.value = props.section.bodyMarkdown.trim() ? 'read' : 'write'
  },
  { immediate: true }
)

onUnmounted(() => {
  if (timer) void save()
})
</script>

<template>
  <section class="flex min-h-0 flex-col" :aria-label="section.title" data-testid="section-pane">
    <header class="flex flex-wrap items-center gap-3 rounded-t-card border border-b-0 border-border bg-surface px-5 py-3">
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
        <p class="mt-0.5 text-small text-text-subtle">
          Edited {{ relativeTime(section.updatedAt) }}
          <span class="px-1 text-border-strong" aria-hidden="true">/</span>
          <span :class="saveState === 'unsaved' ? 'text-warn' : ''" role="status" data-testid="section-save-state">{{ saveLabel }}</span>
        </p>
      </div>
      <div class="flex items-center gap-1">
        <AppButton
          variant="ghost"
          size="sm"
          :disabled="!canMoveUp"
          aria-label="Move section up"
          title="Move up"
          data-testid="section-move-up"
          @click="emit('move', 'up')"
        >
          <svg class="size-3.5" viewBox="0 0 14 14" fill="none" aria-hidden="true">
            <path d="M3.5 8.5L7 5l3.5 3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </AppButton>
        <AppButton
          variant="ghost"
          size="sm"
          :disabled="!canMoveDown"
          aria-label="Move section down"
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
      <AppModeToggle v-model="mode" testid-prefix="section-mode" />
    </header>
    <div class="rounded-b-card border border-border bg-surface p-4">
      <Transition name="fade-quick" mode="out-in">
        <AppMarkdownEditor
          v-if="mode === 'write'"
          key="write"
          v-model="bodyDraft"
          :height="editorMinimum"
          grow
          :path-root="repoFolder"
          placeholder="Write in markdown. Anything about this project that isn't the blueprint."
          @update:model-value="schedule"
        />
        <AppMarkdownEditor v-else key="read" :model-value="bodyDraft" readonly />
      </Transition>
    </div>
  </section>
</template>
