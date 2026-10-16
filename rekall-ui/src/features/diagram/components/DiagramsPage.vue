<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import AppCatalogHeader from '@/features/catalog/components/AppCatalogHeader.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import DiagramGallery from './gallery/DiagramGallery.vue'
import DiagramViewer from './viewer/DiagramViewer.vue'
import GenerateDiagramDialog from './dialogs/GenerateDiagramDialog.vue'
import ImportDiagramDialog from './dialogs/ImportDiagramDialog.vue'
import SessionDrawer from './dialogs/SessionDrawer.vue'
import { ApiError } from '@/api/client'
import { useConsoleStore } from '@/stores/console.store'
import { useDiagramStore, type GenerationRequest, type PendingGeneration } from '@/features/diagram/store/diagram.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useToastStore } from '@/stores/toast.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useDiagramStream } from '@/features/diagram/composables/useDiagramStream'
import type { DiagramDraft } from '@/features/diagram/model/diagram'
import type { DiagramId, TerminalId } from '@/model/branded'

/**
 * The Diagrams screen: the gallery of the projects in scope, or one diagram on the whole screen.
 * The open diagram is `?diagram=` and a watched session `?session=`, so both survive a reload
 * and the back button returns to the gallery.
 */
const store = useDiagramStore()
const console_ = useConsoleStore()
const terminals = useTerminalStore()
const toast = useToastStore()
const route = useRoute()
const router = useRouter()
const { run, isRunning } = useAsyncAction()

useDiagramStream((event) => void store.applyEvent(event))

const showGenerate = ref(false)
const showImport = ref(false)
const generating = ref(false)
const importing = ref(false)
const importRefusal = ref<string | null>(null)
const confirmingDelete = ref(false)

const diagram = computed(() => store.current)
const projectOf = computed(() => console_.projects.find((project) => project.id === diagram.value?.projectId) ?? null)
const taskOf = computed(() => console_.tasks.find((task) => task.id === diagram.value?.taskId) ?? null)
const siblings = computed(() => store.summaries.filter((summary) => summary.projectId === diagram.value?.projectId && summary.id !== diagram.value?.id))
const endedTerminals = computed<ReadonlySet<string>>(() => new Set(terminals.terminals.filter((terminal) => !terminal.live).map((terminal) => terminal.id)))
const knownTerminals = computed<ReadonlySet<string>>(() => new Set(terminals.terminals.map((terminal) => terminal.id)))
/** A pending generation whose terminal this console no longer knows ended with the app. */
const finished = computed<ReadonlySet<string>>(() => new Set(store.pending.map((generation) => generation.terminalId).filter((id) => endedTerminals.value.has(id) || !knownTerminals.value.has(id))))

const watchedId = computed(() => (typeof route.query.session === 'string' ? (route.query.session as TerminalId) : null))
const watched = computed(() => terminals.terminals.find((terminal) => terminal.id === watchedId.value && terminal.live) ?? null)

function watchSession(id: TerminalId | null): void {
  void router.replace({ query: { ...route.query, session: id ?? undefined } })
}

function show(id: DiagramId | null): void {
  void router.push({ query: { ...route.query, diagram: id ?? undefined } })
}

watch(
  () => route.query.diagram,
  (id) => {
    if (typeof id === 'string') void run(() => store.open(id as DiagramId))
    else store.close()
  },
  { immediate: true }
)

async function generate(asked: GenerationRequest): Promise<void> {
  generating.value = true
  const started = await run(() => store.generate(asked))
  generating.value = false
  if (!started) return
  showGenerate.value = false
  show(null)
  toast.notify('A session is drawing it. It appears here when it is done.', { label: 'Watch', run: () => watchGeneration(started) })
}

function watchGeneration(generation: PendingGeneration): void {
  watchSession(generation.terminalId)
}

async function importDiagram(draft: DiagramDraft): Promise<void> {
  importing.value = true
  importRefusal.value = null
  try {
    const created = await store.importDiagram(draft)
    showImport.value = false
    show(created.id)
    toast.notify('Diagram imported.')
  } catch (caught) {
    importRefusal.value = caught instanceof ApiError ? caught.message : 'The diagram could not be imported.'
  } finally {
    importing.value = false
  }
}

async function removeCurrent(): Promise<void> {
  const current = diagram.value
  confirmingDelete.value = false
  if (!current) return
  const removed = await run(() => store.remove(current.id), 'Diagram deleted.')
  if (removed !== null) show(null)
}

async function stopWatched(): Promise<void> {
  const terminal = watched.value
  if (!terminal) return
  watchSession(null)
  await run(() => terminals.close(terminal.id))
}

onMounted(() => {
  void run(() => store.load())
  void run(() => terminals.load())
})

const blast = computed(() =>
  diagram.value ? `Deletes “${diagram.value.title}” and its ${diagram.value.sheets.length} sheet(s). The code is not touched.` : ''
)
</script>

<template>
  <div class="relative flex min-h-0 flex-1 flex-col bg-canvas">
    <template v-if="!store.selectedId">
      <AppCatalogHeader title="Diagrams">
        <template #actions>
          <AppButton size="sm" variant="secondary" data-testid="diagram-import-open" @click="showImport = true">Import</AppButton>
          <AppButton size="sm" variant="primary" data-testid="diagram-generate-open" @click="showGenerate = true">
            <svg viewBox="0 0 16 16" class="size-3.5" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M8 2.2 9.5 6.5 13.8 8 9.5 9.5 8 13.8 6.5 9.5 2.2 8 6.5 6.5Z" /></svg>
            Generate
          </AppButton>
        </template>
      </AppCatalogHeader>
      <DiagramGallery
        :summaries="store.summaries"
        :projects="console_.scopedProjects"
        :tasks="console_.tasks"
        :pending="store.pending"
        :ended-terminals="finished"
        :arrived-id="store.arrivedId"
        :loaded="store.loaded"
        @open="show"
        @generate="showGenerate = true"
        @import="showImport = true"
        @watch="watchGeneration"
        @dismiss="store.dismissPending"
      />
    </template>

    <main v-else class="relative flex min-h-0 flex-1" aria-label="Diagram">
      <Transition name="pane" mode="out-in">
        <DiagramViewer
          v-if="diagram"
          :key="diagram.id"
          :diagram="diagram"
          :siblings="siblings"
          :project-anchor="projectOf ? `project:${projectOf.label}` : null"
          :task-anchor="taskOf ? `task:${taskOf.label}` : null"
          @back="show(null)"
          @open="show"
          @remove="confirmingDelete = true"
        />
        <div v-else key="opening" class="grid flex-1 place-items-center" role="status" aria-busy="true">
          <span class="sr-only">Opening the diagram</span>
          <span class="skeleton h-40 w-[min(520px,70%)] rounded-card" />
        </div>
      </Transition>
    </main>

    <Transition name="fade-quick">
      <SessionDrawer v-if="watched" :key="watched.id" :terminal="watched" @close="watchSession(null)" @stop="stopWatched" />
    </Transition>

    <Transition name="dialog">
      <GenerateDiagramDialog
        v-if="showGenerate"
        :projects="console_.scopedProjects.length > 0 ? console_.scopedProjects : console_.projects"
        :tasks="console_.tasks"
        :initial-project-id="diagram?.projectId ?? console_.scopedProject?.id ?? null"
        :initial-task-id="diagram?.taskId ?? null"
        :sending="generating || isRunning"
        @cancel="showGenerate = false"
        @generate="generate"
      />
    </Transition>
    <Transition name="dialog">
      <ImportDiagramDialog
        v-if="showImport"
        :projects="console_.scopedProjects.length > 0 ? console_.scopedProjects : console_.projects"
        :tasks="console_.tasks"
        :initial-project-id="diagram?.projectId ?? console_.scopedProject?.id ?? null"
        :sending="importing"
        :refusal="importRefusal"
        @cancel="(showImport = false), (importRefusal = null)"
        @import="importDiagram"
      />
    </Transition>
    <Transition name="dialog">
      <AppConfirm
        v-if="confirmingDelete"
        title="Delete this diagram?"
        body="The diagram goes; nothing else does. A session can generate it again."
        :blast="blast"
        confirm-label="Delete diagram"
        @cancel="confirmingDelete = false"
        @confirm="removeCurrent"
      />
    </Transition>
  </div>
</template>
