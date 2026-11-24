<script setup lang="ts">
import { computed, ref } from 'vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import ProjectSectionPane from '@/features/project/components/ProjectSectionPane.vue'
import ProjectSectionRail from '@/features/project/components/ProjectSectionRail.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { pageNamed, type ReferencePage } from '@/common/markdown/page-reference'
import {
  canMovePage,
  contextMoved,
  layoutOf,
  pageMovedToContext,
  pageMovedWithin,
  sameLayout
} from '@/features/project/lib/section-layout'
import type { Direction } from '@/features/project/lib/section-order'
import { sectionOfFile, type SectionFile } from '@/features/project/lib/section-file'
import { useProjectSectionsStore } from '@/features/project/store/projectSections.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useToastStore } from '@/stores/toast.store'
import type { Project } from '@/features/catalog/model/catalog'
import type { ProjectSection } from '@/features/project/model/project-section'
import type { SectionContext } from '@/features/project/model/section-context'
import type { ProjectSectionId, SectionContextId } from '@/model/branded'

/** The project's pages: their index, grouped in contexts, beside the one open, written, moved and deleted in place. */
const props = defineProps<{ project: Project; selectedId: ProjectSectionId | null }>()

const emit = defineEmits<{ open: [id: ProjectSectionId | null] }>()

const store = useProjectSectionsStore()
const terminals = useTerminalStore()
const toast = useToastStore()
const { run } = useAsyncAction()

const sections = computed(() => store.sectionsOf(props.project.id))
const contexts = computed(() => store.contextsOf(props.project.id))
const layout = computed(() => layoutOf(contexts.value, sections.value))
const pages = computed<ReferencePage[]>(() =>
  sections.value.map((section) => ({
    title: section.title,
    contextTitle: contexts.value.find((context) => context.id === section.contextId)?.title ?? '',
    characters: section.bodyMarkdown.length
  }))
)
const selected = computed(() => sections.value.find((section) => section.id === props.selectedId) ?? null)
const creating = ref(false)
const deleting = ref<ProjectSection | null>(null)
const deletingContext = ref<SectionContext | null>(null)
const autoconfiguring = ref(false)
const reorganizing = ref(false)
const fileInput = ref<HTMLInputElement | null>(null)
const importingAsNew = ref(true)
const replacing = ref<{ section: ProjectSection; file: SectionFile } | null>(null)

async function create(title: string, contextId: SectionContextId | null): Promise<void> {
  creating.value = true
  const created = await run(() => store.create(props.project.id, title, '', contextId ?? undefined))
  creating.value = false
  if (created) emit('open', created.id)
}

function chooseFile(asNew: boolean): void {
  importingAsNew.value = asNew
  fileInput.value?.click()
}

async function fileChosen(event: Event): Promise<void> {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file) return
  const asNew = importingAsNew.value
  const target = selected.value
  const read = await run(async () => sectionOfFile(file.name, await file.text()))
  if (!read) return
  if (asNew || !target) await importAsNew(read)
  else replacing.value = { section: target, file: read }
}

async function importAsNew(file: SectionFile): Promise<void> {
  const created = await run(() => store.create(props.project.id, file.title, file.body), `Imported ${file.title}`)
  if (created) emit('open', created.id)
}

async function confirmReplace(): Promise<void> {
  const pending = replacing.value
  replacing.value = null
  if (!pending) return
  await run(() => store.update(pending.section.id, { bodyMarkdown: pending.file.body }), `Replaced the text of ${pending.section.title}`)
}

async function autoconfigure(): Promise<void> {
  autoconfiguring.value = true
  const terminal = await run(() => store.autoconfigure(props.project.id))
  autoconfiguring.value = false
  if (!terminal) return
  if (terminal.kind === 'TERMINAL') terminals.select(terminal.id)
  toast.notify(`A session is learning how ${props.project.title} takes a contribution.`)
}

async function reorganize(): Promise<void> {
  reorganizing.value = true
  const terminal = await run(() => store.reorganize(props.project.id))
  reorganizing.value = false
  if (!terminal) return
  if (terminal.kind === 'TERMINAL') terminals.select(terminal.id)
  toast.notify(`A session is regrouping the pages of ${props.project.title} by concept.`)
}

async function arrange(placements: ReturnType<typeof layoutOf>): Promise<void> {
  if (sameLayout(placements, layout.value)) return
  await run(() => store.arrange(props.project.id, placements))
}

async function move(direction: Direction): Promise<void> {
  if (selected.value) await arrange(pageMovedWithin(layout.value, selected.value.id, direction))
}

async function movePage(id: ProjectSectionId, contextId: SectionContextId, beforeId: ProjectSectionId | null): Promise<void> {
  await arrange(pageMovedToContext(layout.value, id, contextId, beforeId))
}

async function moveSelectedToContext(contextId: SectionContextId): Promise<void> {
  if (selected.value) await movePage(selected.value.id, contextId, null)
}

async function createContext(title: string): Promise<SectionContext | null> {
  return run(() => store.createContext(props.project.id, title))
}

async function createContextForSelected(title: string): Promise<void> {
  const created = await createContext(title)
  if (created) await moveSelectedToContext(created.id)
}

async function renameContext(id: SectionContextId, title: string): Promise<void> {
  await run(() => store.renameContext(id, title))
}

async function moveContext(id: SectionContextId, direction: Direction): Promise<void> {
  await arrange(contextMoved(layout.value, id, direction))
}

async function confirmDeleteContext(): Promise<void> {
  const target = deletingContext.value
  deletingContext.value = null
  if (!target) return
  await run(() => store.removeContext(props.project.id, target.id).then(() => true), `Deleted the context ${target.title}`)
}

function openPageTitled(title: string): void {
  const page = pageNamed(pages.value, title)
  const found = page && sections.value.find((section) => section.title === page.title)
  if (found) emit('open', found.id)
}

async function confirmDelete(): Promise<void> {
  const target = deleting.value
  deleting.value = null
  if (!target) return
  const removed = await run(() => store.remove(props.project.id, target.id).then(() => true), `Deleted ${target.title}`)
  if (removed) emit('open', sections.value[0]?.id ?? null)
}
</script>

<template>
  <div class="grid h-full min-h-0 grid-cols-1 gap-4 md:grid-cols-[232px_minmax(0,1fr)]" data-testid="sections-page">
    <ProjectSectionRail
      class="max-h-[30vh] md:max-h-none"
      :contexts="contexts"
      :sections="sections"
      :selected-id="selected?.id ?? null"
      :loading="store.isLoading(project.id)"
      :creating="creating"
      :autoconfiguring="autoconfiguring"
      :reorganizing="reorganizing"
      @select="emit('open', $event)"
      @create="create"
      @create-context="createContext"
      @rename-context="renameContext"
      @move-context="moveContext"
      @delete-context="deletingContext = contexts.find((context) => context.id === $event) ?? null"
      @move-page="movePage"
      @autoconfigure="autoconfigure"
      @reorganize="reorganize"
      @import="chooseFile(true)"
    />
    <input ref="fileInput" type="file" accept=".md,.markdown,.txt,text/markdown,text/plain" class="hidden" data-testid="section-file" @change="fileChosen" />

    <Transition name="fade-quick" mode="out-in">
      <ProjectSectionPane
        v-if="selected"
        :key="selected.id"
        :section="selected"
        :contexts="contexts"
        :sections="sections"
        :pages="pages"
        :repo-folder="project.repoFolder"
        :can-move-up="canMovePage(layout, selected.id, 'up')"
        :can-move-down="canMovePage(layout, selected.id, 'down')"
        @move="move"
        @move-to-context="moveSelectedToContext"
        @create-context="createContextForSelected"
        @open-page="openPageTitled"
        @delete="deleting = selected"
        @replace="chooseFile(false)"
      />
      <div
        v-else
        key="none"
        class="grid h-full min-h-[240px] place-items-center rounded-card border border-dashed border-border px-6 text-center"
        data-testid="sections-none"
      >
        <p class="max-w-[44ch] text-small leading-relaxed text-text-subtle">
          {{ sections.length ? 'Pick a page to read or write it.' : 'Start one with +, or import a markdown file, to keep what the blueprint does not hold.' }}
        </p>
      </div>
    </Transition>

    <Transition name="dialog">
      <AppConfirm
        v-if="replacing"
        :title="`Replace the text of ${replacing.section.title}?`"
        body="The file's text takes the place of what is written. No history keeps it."
        :blast="`Overwrites the text of the page “${replacing.section.title}” in ${project.title}.`"
        confirm-label="Replace text"
        @cancel="replacing = null"
        @confirm="confirmReplace"
      />
    </Transition>

    <Transition name="dialog">
      <AppConfirm
        v-if="deleting"
        :title="`Delete ${deleting.title}?`"
        body="The page and everything written in it is removed. No history keeps it."
        :blast="`Deletes the page “${deleting.title}” from ${project.title}.`"
        confirm-label="Delete page"
        @cancel="deleting = null"
        @confirm="confirmDelete"
      />
    </Transition>

    <Transition name="dialog">
      <AppConfirm
        v-if="deletingContext"
        :title="`Delete the context ${deletingContext.title}?`"
        body="The context is empty, so no page goes with it."
        :blast="`Deletes the context “${deletingContext.title}” from ${project.title}.`"
        confirm-label="Delete context"
        @cancel="deletingContext = null"
        @confirm="confirmDeleteContext"
      />
    </Transition>
  </div>
</template>
