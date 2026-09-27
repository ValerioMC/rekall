import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import {
  createDiagram,
  deleteDiagram,
  fetchDiagram,
  fetchDiagrams
} from '@/api/diagrams.api'
import { preferredEffort, preferredModel, skipsPermissions } from '@/common/config/claude-launch'
import { useTerminalStore } from '@/stores/terminal.store'
import type { Diagram, DiagramDraft, DiagramStreamEvent, DiagramSummary } from '@/model/diagram'
import type { DiagramId, ProjectId, TaskId, TerminalId } from '@/model/branded'

/** A generation asked for and not yet answered: a session is working on it in a terminal. */
export interface PendingGeneration {
  readonly key: number
  readonly projectId: ProjectId
  readonly taskId: TaskId
  readonly request: string
  readonly terminalId: TerminalId
  readonly startedAt: number
}

/**
 * The diagram library and the one diagram on screen. Summaries are the list; a full diagram
 * is fetched when opened and kept, since a graph does not change unless an event says so.
 */
export const useDiagramStore = defineStore('diagram', () => {
  const summaries = ref<DiagramSummary[]>([])
  const cache = ref<Readonly<Record<string, Diagram>>>({})
  const selectedId = ref<DiagramId | null>(null)
  const loaded = ref(false)
  const opening = ref(false)
  const pending = ref<PendingGeneration[]>([])
  /** The last diagram a session delivered, so the library can mark its arrival once. */
  const arrivedId = ref<DiagramId | null>(null)
  let nextPendingKey = 0

  const current = computed<Diagram | null>(() => (selectedId.value ? cache.value[selectedId.value] ?? null : null))

  async function load(): Promise<void> {
    summaries.value = await fetchDiagrams()
    loaded.value = true
    const first = summaries.value[0]
    if (selectedId.value === null && first) await select(first.id)
  }

  async function select(id: DiagramId): Promise<void> {
    selectedId.value = id
    if (cache.value[id]) return
    opening.value = true
    try {
      const diagram = await fetchDiagram(id)
      cache.value = { ...cache.value, [id]: diagram }
    } finally {
      opening.value = false
    }
  }

  async function remove(id: DiagramId): Promise<void> {
    await deleteDiagram(id)
    forget(id)
  }

  async function importDiagram(draft: DiagramDraft): Promise<Diagram> {
    const created = await createDiagram(draft)
    upsertSummary(created)
    cache.value = { ...cache.value, [created.id]: created }
    selectedId.value = created.id
    return created
  }

  /**
   * Starts the generation workflow: a terminal on the task, opened on
   * `/rk <anchors> generate "<request>"`. The diagram comes back through the event stream.
   */
  async function generate(projectId: ProjectId, taskId: TaskId, request: string): Promise<PendingGeneration> {
    const terminal = await useTerminalStore().openForTask(taskId, {
      skipPermissions: skipsPermissions(),
      model: preferredModel(),
      effort: preferredEffort(),
      mode: 'GENERATE',
      request
    })
    const generation: PendingGeneration = {
      key: nextPendingKey++,
      projectId,
      taskId,
      request,
      terminalId: terminal.id,
      startedAt: Date.now()
    }
    pending.value = [generation, ...pending.value]
    return generation
  }

  function dismissPending(key: number): void {
    pending.value = pending.value.filter((generation) => generation.key !== key)
  }

  /** A diagram written or deleted anywhere: a session, another window, this one. */
  async function applyEvent(event: DiagramStreamEvent): Promise<void> {
    if (event.deleted || !event.diagram) {
      forget(event.diagramId)
      return
    }
    const summary = event.diagram
    const isNew = !summaries.value.some((known) => known.id === summary.id)
    upsertSummary(summary)
    cache.value = without(cache.value, summary.id)
    const answered = pending.value.find((generation) => generation.projectId === summary.projectId)
    if (answered) {
      dismissPending(answered.key)
      arrivedId.value = summary.id
    }
    if (selectedId.value === summary.id || (isNew && answered)) await select(summary.id)
  }

  function upsertSummary(summary: DiagramSummary): void {
    const rest = summaries.value.filter((known) => known.id !== summary.id)
    summaries.value = [toSummary(summary), ...rest]
  }

  function forget(id: DiagramId): void {
    summaries.value = summaries.value.filter((known) => known.id !== id)
    cache.value = without(cache.value, id)
    if (selectedId.value === id) {
      selectedId.value = null
      const next = summaries.value[0]
      if (next) void select(next.id)
    }
  }

  return {
    summaries,
    selectedId,
    current,
    loaded,
    opening,
    pending,
    arrivedId,
    load,
    select,
    remove,
    importDiagram,
    generate,
    dismissPending,
    applyEvent
  }
})

function without(cache: Readonly<Record<string, Diagram>>, id: string): Record<string, Diagram> {
  const kept = { ...cache }
  delete kept[id]
  return kept
}

function toSummary(diagram: DiagramSummary): DiagramSummary {
  return {
    id: diagram.id,
    projectId: diagram.projectId,
    taskId: diagram.taskId,
    title: diagram.title,
    question: diagram.question,
    nodeCount: diagram.nodeCount,
    edgeCount: diagram.edgeCount,
    createdAt: diagram.createdAt,
    updatedAt: diagram.updatedAt
  }
}
