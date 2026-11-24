<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import ProjectSectionContextGroup, { type DropTarget } from '@/features/project/components/ProjectSectionContextGroup.vue'
import { canMoveContext, layoutOf } from '@/features/project/lib/section-layout'
import type { Direction } from '@/features/project/lib/section-order'
import type { ProjectSection } from '@/features/project/model/project-section'
import type { SectionContext } from '@/features/project/model/section-context'
import type { ProjectSectionId, SectionContextId } from '@/model/branded'

/** The project's pages as an index beside the one open, grouped in their contexts, with the fields to start a page or a context. */
const props = defineProps<{
  contexts: readonly SectionContext[]
  sections: readonly ProjectSection[]
  selectedId: ProjectSectionId | null
  loading: boolean
  creating: boolean
  autoconfiguring: boolean
  reorganizing: boolean
}>()

const emit = defineEmits<{
  select: [id: ProjectSectionId]
  create: [title: string, contextId: SectionContextId | null]
  createContext: [title: string]
  renameContext: [id: SectionContextId, title: string]
  moveContext: [id: SectionContextId, direction: Direction]
  deleteContext: [id: SectionContextId]
  movePage: [id: ProjectSectionId, contextId: SectionContextId, beforeId: ProjectSectionId | null]
  autoconfigure: []
  reorganize: []
  import: []
}>()

const collapsed = ref<ReadonlySet<SectionContextId>>(new Set())
const composingIn = ref<SectionContextId | null>(null)
const composingFirstPage = ref(false)
const composingContext = ref(false)
const contextTitle = ref('')
const firstPageTitle = ref('')
const contextInput = ref<HTMLInputElement | null>(null)
const firstPageInput = ref<HTMLInputElement | null>(null)
const draggedId = ref<ProjectSectionId | null>(null)
const dropTarget = ref<DropTarget | null>(null)

const layout = computed(() => layoutOf(props.contexts, props.sections))
const selectedContextId = computed(() => props.sections.find((section) => section.id === props.selectedId)?.contextId ?? null)
const nothingYet = computed(() => props.contexts.length === 0 && props.sections.length === 0)

function sectionsIn(context: SectionContext): ProjectSection[] {
  return props.sections.filter((section) => section.contextId === context.id)
}

function isOpen(context: SectionContext): boolean {
  return !collapsed.value.has(context.id)
}

function toggle(context: SectionContext): void {
  const next = new Set(collapsed.value)
  if (!next.delete(context.id)) next.add(context.id)
  collapsed.value = next
}

function reveal(contextId: SectionContextId): void {
  const next = new Set(collapsed.value)
  next.delete(contextId)
  collapsed.value = next
}

async function startNewPage(): Promise<void> {
  const target = selectedContextId.value ?? props.contexts[0]?.id ?? null
  if (target === null) {
    composingFirstPage.value = true
    await nextTick()
    firstPageInput.value?.focus()
    return
  }
  reveal(target)
  composingIn.value = target
}

function submitPage(contextId: SectionContextId, title: string): void {
  if (props.creating) return
  composingIn.value = null
  emit('create', title, contextId)
}

function submitFirstPage(): void {
  const wanted = firstPageTitle.value.trim()
  if (!wanted || props.creating) return
  emit('create', wanted, null)
  stopFirstPage()
}

function stopFirstPage(): void {
  composingFirstPage.value = false
  firstPageTitle.value = ''
}

async function startNewContext(): Promise<void> {
  composingContext.value = true
  await nextTick()
  contextInput.value?.focus()
}

function submitContext(): void {
  const wanted = contextTitle.value.trim()
  if (!wanted) return
  emit('createContext', wanted)
  stopNewContext()
}

function stopNewContext(): void {
  composingContext.value = false
  contextTitle.value = ''
}

function beginDrag(id: ProjectSectionId, event: DragEvent): void {
  draggedId.value = id
  if (!event.dataTransfer) return
  event.dataTransfer.effectAllowed = 'move'
  event.dataTransfer.setData('text/plain', id)
}

function endDrag(): void {
  draggedId.value = null
  dropTarget.value = null
}

function drop(): void {
  const moved = draggedId.value
  const target = dropTarget.value
  endDrag()
  if (moved !== null && target !== null) emit('movePage', moved, target.contextId, target.beforeId)
}
</script>

<template>
  <nav class="flex min-h-0 flex-col gap-1 overflow-y-auto" aria-label="Pages" data-testid="project-section-rail">
    <div class="flex items-center justify-between pl-4 pr-1">
      <p class="text-small font-semibold text-text-muted">Pages<span v-if="sections.length" class="ml-1.5 font-mono text-micro font-normal text-text-subtle">{{ sections.length }}</span></p>
      <span class="flex-1" />
      <button
        type="button"
        class="key-quiet focus-ring grid size-6 place-items-center rounded-key"
        aria-label="Autoconfigure from the repository"
        title="Autoconfigure: a session reads the repository and writes how to contribute"
        :disabled="autoconfiguring"
        data-testid="rail-autoconfigure"
        @click="emit('autoconfigure')"
      >
        <svg class="size-3.5" viewBox="0 0 14 14" fill="none" aria-hidden="true">
          <path d="M7 1.5 8.2 5.8 12.5 7 8.2 8.2 7 12.5 5.8 8.2 1.5 7 5.8 5.8Z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
        </svg>
      </button>
      <button
        type="button"
        class="key-quiet focus-ring grid size-6 place-items-center rounded-key"
        aria-label="Import a markdown file"
        title="Import a .md file as a page"
        data-testid="rail-import"
        @click="emit('import')"
      >
        <svg class="size-3.5" viewBox="0 0 14 14" fill="none" aria-hidden="true">
          <path d="M7 9V2M4.5 4.3 7 1.8l2.5 2.5M2 8.8v1.7a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1V8.8" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </button>
      <button
        type="button"
        class="key-quiet focus-ring grid size-6 place-items-center rounded-key"
        :class="composingContext && 'bg-surface-raised text-text'"
        aria-label="New context"
        title="New context: a group of pages"
        data-testid="rail-new-context"
        @click="startNewContext"
      >
        <svg class="size-3.5" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <path d="M2 4.6A1.1 1.1 0 0 1 3.1 3.5h2.7l1.4 1.6h5.7A1.1 1.1 0 0 1 14 6.2v5.9a1.1 1.1 0 0 1-1.1 1.1H3.1A1.1 1.1 0 0 1 2 12.1Z M8 7.6v3.2M6.4 9.2h3.2" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </button>
      <button
        type="button"
        class="key-quiet focus-ring grid size-6 place-items-center rounded-key"
        aria-label="New page"
        title="New page"
        data-testid="rail-new-section"
        @click="startNewPage"
      >
        <svg class="size-3.5" viewBox="0 0 14 14" fill="none" aria-hidden="true">
          <path d="M7 2.5v9M2.5 7h9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
      </button>
    </div>

    <Transition name="detail">
      <form v-if="composingContext" class="px-1" data-testid="rail-new-context-form" @submit.prevent="submitContext">
        <label class="sr-only" for="new-context-title">Context title</label>
        <input
          id="new-context-title"
          ref="contextInput"
          v-model="contextTitle"
          maxlength="200"
          autocomplete="off"
          placeholder="Context, then Enter"
          class="field text-text h-(--spacing-control) w-full rounded-control px-3 text-body"
          data-testid="rail-new-context-title"
          @keydown.esc.stop.prevent="stopNewContext"
          @blur="contextTitle.trim() === '' && stopNewContext()"
        />
        <p class="px-1 pt-1.5 text-caption text-text-subtle">Enter adds an empty context. Esc cancels.</p>
      </form>
    </Transition>

    <Transition name="detail">
      <form v-if="composingFirstPage" class="px-1" data-testid="rail-new-form" @submit.prevent="submitFirstPage">
        <label class="sr-only" for="new-section-title">Page title</label>
        <input
          id="new-section-title"
          ref="firstPageInput"
          v-model="firstPageTitle"
          maxlength="200"
          autocomplete="off"
          placeholder="Title, then Enter"
          class="field text-text h-(--spacing-control) w-full rounded-control px-3 text-body"
          data-testid="rail-new-title"
          @keydown.esc.stop.prevent="stopFirstPage"
          @blur="firstPageTitle.trim() === '' && stopFirstPage()"
        />
      </form>
    </Transition>

    <div v-if="loading && nothingYet" class="flex flex-col gap-1.5 px-1 pt-1" role="status" aria-live="polite" aria-busy="true">
      <span class="sr-only">Loading pages</span>
      <div v-for="row in 2" :key="row" class="skeleton h-(--spacing-row) rounded-control" />
    </div>

    <div v-else class="flex flex-col gap-3">
      <ProjectSectionContextGroup
        v-for="context in contexts"
        :key="context.id"
        :context="context"
        :sections="sectionsIn(context)"
        :selected-id="selectedId"
        :open="isOpen(context)"
        :composing="composingIn === context.id"
        :can-move-up="canMoveContext(layout, context.id, 'up')"
        :can-move-down="canMoveContext(layout, context.id, 'down')"
        :dragged-id="draggedId"
        :drop-target="dropTarget"
        @toggle="toggle(context)"
        @select="emit('select', $event)"
        @compose="reveal(context.id); composingIn = context.id"
        @composed="submitPage(context.id, $event)"
        @compose-cancel="composingIn = null"
        @rename="emit('renameContext', context.id, $event)"
        @move-up="emit('moveContext', context.id, 'up')"
        @move-down="emit('moveContext', context.id, 'down')"
        @delete="emit('deleteContext', context.id)"
        @drag-start="beginDrag"
        @drag-end="endDrag"
        @drag-over="dropTarget = $event"
        @drop="drop"
      />
    </div>

    <p
      v-if="!loading && nothingYet && !composingFirstPage && !composingContext"
      class="px-4 pt-1 text-small leading-relaxed text-text-subtle"
      data-testid="rail-empty"
    >
      Keep what the blueprint doesn't hold, grouped by what it is about: deployment, conventions, a glossary.
    </p>

    <div v-if="!loading && sections.length > 0" class="px-1 pt-2">
      <AppButton
        variant="quiet"
        size="sm"
        class="w-full justify-start"
        :disabled="reorganizing || sections.length < 2"
        :title="sections.length < 2 ? 'Reorganize needs at least two pages' : 'A session reads every page and regroups them in contexts by concept'"
        data-testid="rail-reorganize"
        @click="emit('reorganize')"
      >
        <svg class="size-3.5" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <path d="M2.5 4.5h7M2.5 8h11M2.5 11.5h5M11.5 3l2 1.5-2 1.5M9.5 10l2 1.5 2-1.5" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        Reorganize
      </AppButton>
    </div>

    <p class="mt-auto border-t border-border px-4 pt-3 text-caption leading-relaxed text-text-subtle">
      Pages stay out of Claude's context. A context lists their titles, and a session reads one with
      <code class="text-text-muted">rekall_read_sections</code> when the work touches it.
    </p>
  </nav>
</template>
