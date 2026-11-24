<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'
import PopoverMenu, { type PopoverMenuEntry } from '@/components/ui/PopoverMenu.vue'
import ProjectSectionPageRow from '@/features/project/components/ProjectSectionPageRow.vue'
import type { ProjectSection } from '@/features/project/model/project-section'
import type { SectionContext } from '@/features/project/model/section-context'
import type { ProjectSectionId, SectionContextId } from '@/model/branded'

/** Where a dragged page would land: in this context, before one of its pages or last. */
export interface DropTarget {
  readonly contextId: SectionContextId
  readonly beforeId: ProjectSectionId | null
}

/** One context of the rail: its name and count, what a person does to it, and the pages it holds. */
const props = defineProps<{
  context: SectionContext
  sections: readonly ProjectSection[]
  selectedId: ProjectSectionId | null
  open: boolean
  composing: boolean
  canMoveUp: boolean
  canMoveDown: boolean
  draggedId: ProjectSectionId | null
  dropTarget: DropTarget | null
}>()

const emit = defineEmits<{
  toggle: []
  select: [id: ProjectSectionId]
  compose: []
  composed: [title: string]
  composeCancel: []
  rename: [title: string]
  moveUp: []
  moveDown: []
  delete: []
  dragStart: [id: ProjectSectionId, event: DragEvent]
  dragEnd: []
  dragOver: [target: DropTarget]
  drop: []
}>()

const optionsOpen = ref(false)
const renaming = ref(false)
const renameDraft = ref('')
const pageTitle = ref('')
const renameInput = ref<HTMLInputElement | null>(null)

const overThisContext = computed(() => props.dropTarget?.contextId === props.context.id)
const optionEntries = computed<readonly PopoverMenuEntry[]>(() => [
  { kind: 'item', id: 'rename', label: 'Rename' },
  { kind: 'item', id: 'up', label: 'Move context up', disabled: !props.canMoveUp },
  { kind: 'item', id: 'down', label: 'Move context down', disabled: !props.canMoveDown },
  { kind: 'separator' },
  {
    kind: 'item',
    id: 'delete',
    label: 'Delete context',
    tone: 'danger',
    disabled: props.sections.length > 0,
    ...(props.sections.length > 0 ? { hint: `Move its ${props.sections.length} ${props.sections.length === 1 ? 'page' : 'pages'} out first.` } : {})
  }
])

async function startRename(): Promise<void> {
  renameDraft.value = props.context.title
  renaming.value = true
  await nextTick()
  renameInput.value?.select()
}

function saveRename(): void {
  const wanted = renameDraft.value.trim()
  renaming.value = false
  if (wanted !== '' && wanted !== props.context.title) emit('rename', wanted)
}

function choose(id: string): void {
  if (id === 'rename') void startRename()
  else if (id === 'up') emit('moveUp')
  else if (id === 'down') emit('moveDown')
  else emit('delete')
}

function submitPage(): void {
  const wanted = pageTitle.value.trim()
  if (!wanted) return
  emit('composed', wanted)
  pageTitle.value = ''
}

function cancelPage(): void {
  pageTitle.value = ''
  emit('composeCancel')
}

function focusPageInput(element: unknown): void {
  if (element instanceof HTMLInputElement) void nextTick(() => element.focus())
}

function overPage(event: DragEvent, id: ProjectSectionId): void {
  if (props.draggedId === null) return
  event.preventDefault()
  emit('dragOver', { contextId: props.context.id, beforeId: id })
}

function overList(event: DragEvent): void {
  if (props.draggedId === null) return
  event.preventDefault()
  emit('dragOver', { contextId: props.context.id, beforeId: null })
}
</script>

<template>
  <section class="flex flex-col" :aria-label="context.title" :data-testid="`rail-context-${context.id}`">
    <div
      class="group/head flex items-center rounded-control transition-colors duration-(--motion-quick)"
      :class="overThisContext && draggedId !== null ? 'bg-accent-soft outline outline-1 -outline-offset-1 outline-accent-deep' : ''"
      @dragover="overList"
      @drop.prevent="emit('drop')"
    >
      <form v-if="renaming" class="min-w-0 flex-1 px-1" @submit.prevent="saveRename">
        <label class="sr-only" :for="`rename-${context.id}`">Context title</label>
        <input
          :id="`rename-${context.id}`"
          ref="renameInput"
          v-model="renameDraft"
          maxlength="200"
          autocomplete="off"
          class="field text-text h-7 w-full rounded-control px-2 text-small font-semibold"
          :data-testid="`rail-context-rename-${context.id}`"
          @keydown.esc.stop.prevent="renaming = false"
          @blur="saveRename"
        />
      </form>
      <button
        v-else
        type="button"
        class="focus-ring flex h-7 min-w-0 flex-1 cursor-pointer items-center gap-1.5 rounded-control px-1.5 text-left text-small font-semibold text-text-muted transition-colors duration-(--motion-quick) hover:bg-surface-raised hover:text-text"
        :aria-expanded="open"
        :data-testid="`rail-context-toggle-${context.id}`"
        @click="emit('toggle')"
      >
        <svg class="size-3 shrink-0 text-text-subtle transition-transform duration-(--motion-quick)" :class="open ? '' : '-rotate-90'" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <path d="m4 6 4 4 4-4" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        <span class="truncate">{{ context.title }}</span>
        <span class="font-mono text-micro font-normal text-text-subtle">{{ sections.length }}</span>
      </button>
      <span
        v-if="!renaming"
        class="flex shrink-0 opacity-0 transition-opacity duration-(--motion-quick) focus-within:opacity-100 group-hover/head:opacity-100"
        :class="optionsOpen && 'opacity-100'"
      >
        <button
          type="button"
          class="key-quiet focus-ring grid size-6 place-items-center rounded-key"
          :aria-label="`New page in ${context.title}`"
          :title="`New page in ${context.title}`"
          :data-testid="`rail-context-new-page-${context.id}`"
          @click="emit('compose')"
        >
          <svg class="size-3.5" viewBox="0 0 14 14" fill="none" aria-hidden="true">
            <path d="M7 2.5v9M2.5 7h9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
          </svg>
        </button>
        <PopoverMenu v-model:open="optionsOpen" :entries="optionEntries" :label="`${context.title} options`" align="right" @pick="choose">
          <template #trigger="{ toggle }">
            <button
              type="button"
              class="key-quiet focus-ring grid size-6 place-items-center rounded-key"
              :aria-label="`${context.title} options`"
              aria-haspopup="menu"
              :aria-expanded="optionsOpen"
              data-popover-trigger
              :data-testid="`rail-context-options-${context.id}`"
              @click="toggle"
            >
              <svg class="size-3.5" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
                <circle cx="3.5" cy="8" r="1.2" /><circle cx="8" cy="8" r="1.2" /><circle cx="12.5" cy="8" r="1.2" />
              </svg>
            </button>
          </template>
        </PopoverMenu>
      </span>
    </div>

    <div v-if="open" class="mt-0.5 ml-3 flex flex-col gap-px border-l border-border pl-1.5" @dragover="overList" @drop.prevent="emit('drop')">
      <template v-for="section in sections" :key="section.id">
        <div v-if="dropTarget?.contextId === context.id && dropTarget.beforeId === section.id && draggedId !== section.id" class="mx-2.5 h-0.5 rounded-full bg-accent" aria-hidden="true" />
        <div @dragover.stop="overPage($event, section.id)" @drop.prevent.stop="emit('drop')">
          <ProjectSectionPageRow
            :section="section"
            :selected="selectedId === section.id"
            :dragged="draggedId === section.id"
            @select="emit('select', section.id)"
            @drag-start="emit('dragStart', section.id, $event)"
            @drag-end="emit('dragEnd')"
          />
        </div>
      </template>
      <div v-if="dropTarget?.contextId === context.id && dropTarget.beforeId === null && draggedId !== null" class="mx-2.5 h-0.5 rounded-full bg-accent" aria-hidden="true" />
      <form v-if="composing" class="px-1 py-1" :data-testid="`rail-context-page-form-${context.id}`" @submit.prevent="submitPage">
        <label class="sr-only" :for="`new-page-${context.id}`">Page title</label>
        <input
          :id="`new-page-${context.id}`"
          :ref="focusPageInput"
          v-model="pageTitle"
          maxlength="200"
          autocomplete="off"
          placeholder="Title, then Enter"
          class="field text-text h-(--spacing-control) w-full rounded-control px-3 text-body"
          :data-testid="`rail-context-page-title-${context.id}`"
          @keydown.esc.stop.prevent="cancelPage"
          @blur="pageTitle.trim() === '' && cancelPage()"
        />
      </form>
      <p v-if="sections.length === 0 && !composing" class="px-2.5 py-1.5 text-caption text-text-subtle" :data-testid="`rail-context-empty-${context.id}`">
        No pages. Drag one here, or add with +.
      </p>
    </div>
  </section>
</template>
