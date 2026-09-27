<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import AppCatalogHeader from '@/components/catalog/AppCatalogHeader.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import DiagramLibrary from '@/components/diagram/DiagramLibrary.vue'
import DiagramOverview from '@/components/diagram/DiagramOverview.vue'
import DiagramStage from '@/components/diagram/DiagramStage.vue'
import GenerateDiagramDialog from '@/components/diagram/GenerateDiagramDialog.vue'
import ImportDiagramDialog from '@/components/diagram/ImportDiagramDialog.vue'
import NodeInspector from '@/components/diagram/NodeInspector.vue'
import { ApiError } from '@/api/client'
import { useConsoleStore } from '@/stores/console.store'
import { useDiagramStore, type PendingGeneration } from '@/stores/diagram.store'
import { useTerminalStore } from '@/stores/terminal.store'
import { useToastStore } from '@/stores/toast.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useDiagramStream } from '@/composables/useDiagramStream'
import { useModalGate } from '@/composables/useModalGate'
import { containerMap, project, type Lens } from '@/common/diagram/graph-projection'
import { layoutProjection, preferredDirection, type Direction } from '@/common/diagram/graph-layout'
import { groupOf, neighbourhoodOf } from '@/common/diagram/neighbourhood'
import { filesOf, normalisePath } from '@/common/diagram/code-index'
import type { Spotlight } from '@/common/diagram/emphasis'
import type { DiagramDraft, NodeKind } from '@/model/diagram'
import type { DiagramId, ProjectId, TaskId } from '@/model/branded'

/**
 * The diagram screen: the library on the left, the drawing in the middle, the inspector on the
 * right. What the reader is looking at (a selection, a file, a kind, a search) is one
 * spotlight, and the canvas only ever draws that.
 */
const store = useDiagramStore()
const console_ = useConsoleStore()
const terminals = useTerminalStore()
const toast = useToastStore()
const router = useRouter()
const { run, isRunning } = useAsyncAction()
const { isModalOpen } = useModalGate()

useDiagramStream((event) => void store.applyEvent(event))

const stage = ref<InstanceType<typeof DiagramStage> | null>(null)
const lens = ref<Lens>('concept')
/** Null until the reader picks one: each diagram opens in the direction that fits it best. */
const chosenDirection = ref<Direction | null>(null)
const collapsed = ref<ReadonlySet<string>>(new Set())
const query = ref('')
const selectedNodeId = ref<string | null>(null)
const focusedFile = ref<string | null>(null)
const hoveredFile = ref<string | null>(null)
const pinnedKind = ref<NodeKind | null>(null)
const hoveredKind = ref<NodeKind | null>(null)
const generating = ref(false)
const importing = ref(false)
const importRefusal = ref<string | null>(null)
const confirmingDelete = ref(false)

const diagram = computed(() => store.current)

const projection = computed(() =>
  diagram.value ? project(diagram.value.graph, { lens: lens.value, collapsed: collapsed.value }) : null
)
const fittedDirection = computed<Direction>(() => {
  const graph = diagram.value?.graph
  return graph ? preferredDirection(project(graph, { lens: 'concept', collapsed: new Set() })) : 'LR'
})
const direction = computed<Direction>({
  get: () => chosenDirection.value ?? fittedDirection.value,
  set: (next) => (chosenDirection.value = next)
})
const layout = computed(() => (projection.value ? layoutProjection(projection.value, direction.value) : null))

/** A selected element folded away or hidden by the lens is shown through what stands for it. */
const visibleSelection = computed(() => {
  const graph = diagram.value?.graph
  const shown = projection.value
  if (!graph || !shown || !selectedNodeId.value) return null
  const visible = new Set(shown.nodes.map((node) => node.id))
  const parents = containerMap(graph)
  let candidate: string | null = selectedNodeId.value
  let stand: string | null = null
  while (candidate !== null) {
    if (visible.has(candidate)) stand = candidate
    candidate = parents.get(candidate) ?? null
  }
  return stand
})

const spotlight = computed<Spotlight | null>(() => {
  const shown = projection.value
  const graph = diagram.value?.graph
  if (!shown || !graph) return null
  if (visibleSelection.value) return { selected: visibleSelection.value, ...neighbourhoodOf(shown, visibleSelection.value) }
  const file = hoveredFile.value ?? focusedFile.value
  if (file) {
    const ids = new Set(filesOf(graph).find((entry) => entry.file === normalisePath(file))?.nodeIds ?? [])
    return { selected: null, ...groupOf(shown, ids) }
  }
  const kind = hoveredKind.value ?? pinnedKind.value
  if (kind) return { selected: null, ...groupOf(shown, new Set(shown.nodes.filter((node) => node.node.kind === kind).map((node) => node.id))) }
  const needle = query.value.trim().toLowerCase()
  if (needle.length >= 2) {
    const ids = shown.nodes
      .filter((node) => node.node.title.toLowerCase().includes(needle) || node.node.id.toLowerCase().includes(needle))
      .map((node) => node.id)
    return { selected: null, ...groupOf(shown, new Set(ids)) }
  }
  return null
})

const selectedNode = computed(() => diagram.value?.graph.nodes.find((node) => node.id === selectedNodeId.value) ?? null)
const selectedProjected = computed(() => projection.value?.nodes.find((node) => node.id === selectedNodeId.value) ?? null)

const projectOf = computed(() => console_.projects.find((candidate) => candidate.id === diagram.value?.projectId) ?? null)
const taskOf = computed(() => console_.tasks.find((candidate) => candidate.id === diagram.value?.taskId) ?? null)

watch(
  () => diagram.value?.id,
  () => {
    selectedNodeId.value = null
    collapsed.value = new Set()
    query.value = ''
    focusedFile.value = null
    pinnedKind.value = null
  }
)

function select(id: string | null): void {
  selectedNodeId.value = id
  if (id) stage.value?.reveal(id)
}

function toggleFold(id: string): void {
  const next = new Set(collapsed.value)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  collapsed.value = next
}

async function openDiagram(id: DiagramId): Promise<void> {
  await run(() => store.select(id))
}

async function generate(projectId: ProjectId, taskId: TaskId, request: string): Promise<void> {
  generating.value = true
  const started = await run(() => store.generate(projectId, taskId, request))
  generating.value = false
  if (!started) return
  showGenerateDialog.value = false
  toast.notify('A session is drawing it. It will appear here when it is done.', {
    label: 'Open terminal',
    run: () => openTerminal(started)
  })
}

function openTerminal(generation: PendingGeneration): void {
  terminals.select(generation.terminalId)
  console_.selectTask(generation.taskId)
  console_.openTerminal()
  void router.push('/')
}

async function importDiagram(draft: DiagramDraft): Promise<void> {
  importing.value = true
  importRefusal.value = null
  try {
    await store.importDiagram(draft)
    showImportDialog.value = false
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
  await run(() => store.remove(current.id), 'Diagram deleted.')
}

const showGenerateDialog = ref(false)
const showImportDialog = ref(false)

function onKeydown(event: KeyboardEvent): void {
  if (isModalOpen.value > 0 || !diagram.value) return
  const target = event.target as HTMLElement | null
  if (target && (/^(INPUT|TEXTAREA|SELECT)$/.test(target.tagName) || target.isContentEditable)) return
  if (event.metaKey || event.ctrlKey || event.altKey) return
  const actions: Record<string, () => void> = {
    f: () => stage.value?.fit(),
    '+': () => stage.value?.zoomIn(),
    '=': () => stage.value?.zoomIn(),
    '-': () => stage.value?.zoomOut(),
    '0': () => stage.value?.resetZoom(),
    '/': () => stage.value?.openSearch(),
    l: () => (lens.value = lens.value === 'concept' ? 'code' : 'concept'),
    d: () => (direction.value = direction.value === 'LR' ? 'TB' : 'LR'),
    Escape: () => {
      selectedNodeId.value = null
      focusedFile.value = null
      pinnedKind.value = null
    }
  }
  const action = actions[event.key]
  if (!action) return
  event.preventDefault()
  action()
}

onMounted(() => {
  window.addEventListener('keydown', onKeydown)
  void run(() => store.load())
})
onUnmounted(() => window.removeEventListener('keydown', onKeydown))

const blast = computed(() =>
  diagram.value
    ? `Deletes “${diagram.value.title}”: ${diagram.value.nodeCount} elements and ${diagram.value.edgeCount} relations. The code is not touched.`
    : ''
)
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col bg-canvas">
    <AppCatalogHeader title="Diagrams">
      <template #actions>
        <AppButton size="sm" variant="secondary" data-testid="diagram-import-open" @click="showImportDialog = true">Import</AppButton>
        <AppButton size="sm" variant="primary" data-testid="diagram-generate-open" @click="showGenerateDialog = true">
          <svg viewBox="0 0 16 16" class="size-3.5" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M8 2.2 9.5 6.5 13.8 8 9.5 9.5 8 13.8 6.5 9.5 2.2 8 6.5 6.5Z" /></svg>
          Generate
        </AppButton>
      </template>
    </AppCatalogHeader>

    <div class="flex min-h-0 flex-1">
      <DiagramLibrary
        :summaries="store.summaries"
        :projects="console_.projects"
        :tasks="console_.tasks"
        :pending="store.pending"
        :selected-id="store.selectedId"
        :arrived-id="store.arrivedId"
        :loaded="store.loaded"
        @select="openDiagram"
        @open-terminal="openTerminal"
        @dismiss="store.dismissPending"
      />

      <main class="relative flex min-h-0 min-w-0 flex-1" aria-label="Diagram">
        <Transition name="pane" mode="out-in">
          <DiagramStage
            v-if="diagram && projection && layout"
            ref="stage"
            :key="diagram.id"
            v-model:lens="lens"
            v-model:direction="direction"
            v-model:query="query"
            :diagram="diagram"
            :projection="projection"
            :layout="layout"
            :spotlight="spotlight"
            :pinned-kind="pinnedKind"
            :collapsed-count="collapsed.size"
            :project-anchor="projectOf ? `project:${projectOf.label}` : null"
            :task-anchor="taskOf ? `task:${taskOf.label}` : null"
            @select="select"
            @toggle-fold="toggleFold"
            @hover-kind="hoveredKind = $event"
            @pin-kind="pinnedKind = $event"
          />
          <div v-else-if="store.opening" key="opening" class="grid flex-1 place-items-center" role="status" aria-busy="true">
            <span class="sr-only">Opening the diagram</span>
            <span class="skeleton h-40 w-[min(520px,70%)] rounded-[var(--radius-card)]" />
          </div>
          <div v-else-if="store.loaded" key="empty" class="grid flex-1 place-items-center p-8">
            <AppEmptyState
              class="max-w-[520px]"
              title="Draw what the code does"
              description="A diagram is a session's reading of the code: the concepts, decisions, states and data behind it, each tied back to the lines that implement it. Ask for one from a task, or import a Semantic Graph."
            >
              <template #icon>
                <svg viewBox="0 0 20 20" fill="none" class="size-5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                  <rect x="2" y="3" width="6" height="4.5" rx="1.4" />
                  <path d="M12 12.5 15 9.5l3 3-3 3Z" />
                  <circle cx="5" cy="15" r="2.2" />
                  <path d="M8 5.2h3.2a2 2 0 0 1 2 2v2.1M7.2 15H11.8" />
                </svg>
              </template>
              <div class="flex gap-2">
                <AppButton size="sm" variant="secondary" @click="showImportDialog = true">Import</AppButton>
                <AppButton size="sm" variant="primary" @click="showGenerateDialog = true">Generate a diagram</AppButton>
              </div>
            </AppEmptyState>
          </div>
        </Transition>
      </main>

      <aside
        v-if="diagram"
        class="w-(--spacing-inspector) shrink-0 overflow-y-auto border-l border-border bg-surface/60 px-5 pb-20 pt-5"
        aria-label="Inspector"
      >
        <Transition name="fade-quick" mode="out-in">
          <NodeInspector
            v-if="selectedNode"
            :key="selectedNode.id"
            :diagram="diagram"
            :node="selectedNode"
            :folded="selectedProjected?.collapsed ?? false"
            :foldable="selectedProjected?.foldable ?? false"
            @select="select"
            @toggle-fold="toggleFold"
          />
          <DiagramOverview
            v-else
            :diagram="diagram"
            :focused-file="focusedFile"
            @focus-file="focusedFile = $event"
            @hover-file="hoveredFile = $event"
            @remove="confirmingDelete = true"
          />
        </Transition>
      </aside>
    </div>

    <Transition name="dialog">
      <GenerateDiagramDialog
        v-if="showGenerateDialog"
        :projects="console_.projects"
        :tasks="console_.tasks"
        :initial-task-id="diagram?.taskId ?? console_.selectedTaskId"
        :sending="generating || isRunning"
        @cancel="showGenerateDialog = false"
        @generate="generate"
      />
    </Transition>
    <Transition name="dialog">
      <ImportDiagramDialog
        v-if="showImportDialog"
        :projects="console_.projects"
        :tasks="console_.tasks"
        :initial-project-id="diagram?.projectId ?? null"
        :sending="importing"
        :refusal="importRefusal"
        @cancel="(showImportDialog = false), (importRefusal = null)"
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
