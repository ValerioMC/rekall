import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { createDiagram, deleteDiagram, fetchDiagram, fetchDiagrams } from '@/api/diagrams.api'
import { preferredEffort, preferredModel, skipsPermissions } from '@/common/config/claude-launch'
import { useTerminalStore } from '@/stores/terminal.store'
import type { Diagram, DiagramDraft, DiagramKind, DiagramStreamEvent, DiagramSummary } from '@/model/diagram'
import type { DiagramId, ProjectId, TaskId, TerminalId } from '@/model/branded'

/** What to generate: a project, optionally one of its tasks, the request and the types asked for. */
export interface GenerationRequest {
  readonly projectId: ProjectId
  readonly taskId: TaskId | null
  readonly request: string
  readonly kinds: readonly DiagramKind[]
}

/** A generation asked for and not yet answered: a session is working on it in a terminal. */
export interface PendingGeneration extends GenerationRequest {
  readonly key: number
  readonly terminalId: TerminalId
  readonly startedAt: number
}

const PENDING_STORAGE_KEY = 'rekall.diagram.pending.v2'

/** Pending generations outlive a reload within the tab; storage may be blocked, then they simply do not. */
function readPending(): PendingGeneration[] {
  try {
    const raw = sessionStorage.getItem(PENDING_STORAGE_KEY)
    return raw ? (JSON.parse(raw) as PendingGeneration[]) : []
  } catch {
    return []
  }
}

function writePending(pending: readonly PendingGeneration[]): void {
  try {
    sessionStorage.setItem(PENDING_STORAGE_KEY, JSON.stringify(pending))
  } catch {
    // Storage is a convenience: without it a reload forgets what was being generated.
  }
}

/**
 * The gallery and the one diagram open. Summaries are the gallery; a full diagram is fetched
 * when opened and kept, since its sheets do not change unless an event says so.
 */
export const useDiagramStore = defineStore('diagram', () => {
  const summaries = ref<DiagramSummary[]>([])
  const cache = ref<Readonly<Record<string, Diagram>>>({})
  const selectedId = ref<DiagramId | null>(null)
  const loaded = ref(false)
  const opening = ref(false)
  const pending = ref<PendingGeneration[]>(readPending())
  /** The last diagram a session delivered, so the gallery can mark its arrival once. */
  const arrivedId = ref<DiagramId | null>(null)
  let nextPendingKey = pending.value.reduce((highest, generation) => Math.max(highest, generation.key + 1), 0)

  const current = computed<Diagram | null>(() => (selectedId.value ? cache.value[selectedId.value] ?? null : null))

  /** Also settles generations whose diagram arrived while no stream was listening. */
  async function load(): Promise<void> {
    summaries.value = await fetchDiagrams()
    loaded.value = true
    settleAnswered(summaries.value)
  }

  async function open(id: DiagramId): Promise<void> {
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

  function close(): void {
    selectedId.value = null
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
   * Starts the generation workflow: a terminal on the task, or on the project alone, opened on
   * `/rk <anchors> generate kinds:… "<request>"`. The diagram comes back through the event stream.
   */
  async function generate(asked: GenerationRequest): Promise<PendingGeneration> {
    const terminals = useTerminalStore()
    const launch = { skipPermissions: skipsPermissions(), model: preferredModel(), effort: preferredEffort(), mode: 'GENERATE' as const, request: asked.request, kinds: asked.kinds }
    const terminal = asked.taskId ? await terminals.openForTask(asked.taskId, launch) : await terminals.openForProject(asked.projectId, launch)
    const generation: PendingGeneration = { ...asked, key: nextPendingKey++, terminalId: terminal.id, startedAt: Date.now() }
    setPending([generation, ...pending.value.filter((known) => known.terminalId !== terminal.id)])
    return generation
  }

  function dismissPending(key: number): void {
    setPending(pending.value.filter((generation) => generation.key !== key))
  }

  function setPending(next: PendingGeneration[]): void {
    pending.value = next
    writePending(next)
  }

  /** A diagram written or deleted anywhere: a session, another window, this one. */
  async function applyEvent(event: DiagramStreamEvent): Promise<void> {
    if (event.deleted || !event.diagram) {
      forget(event.diagramId)
      return
    }
    const summary = event.diagram
    upsertSummary(summary)
    cache.value = without(cache.value, summary.id)
    settleAnswered([summary])
    if (selectedId.value === summary.id) await open(summary.id)
  }

  /** Dismisses every pending generation one of these diagrams answers; the latest generation's answer is marked as arrived. */
  function settleAnswered(candidates: readonly DiagramSummary[]): void {
    const answerOf = (generation: PendingGeneration): DiagramSummary | undefined => candidates.find((summary) => answers(summary, generation))
    const answered = pending.value.filter((generation) => answerOf(generation) !== undefined)
    const latest = answered[0]
    if (!latest) return
    setPending(pending.value.filter((generation) => !answered.includes(generation)))
    arrivedId.value = answerOf(latest)?.id ?? null
  }

  function upsertSummary(summary: DiagramSummary): void {
    const rest = summaries.value.filter((known) => known.id !== summary.id)
    summaries.value = [toSummary(summary), ...rest]
  }

  function forget(id: DiagramId): void {
    summaries.value = summaries.value.filter((known) => known.id !== id)
    cache.value = without(cache.value, id)
    if (selectedId.value === id) selectedId.value = null
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
    open,
    close,
    remove,
    importDiagram,
    generate,
    dismissPending,
    applyEvent
  }
})

/** A generation is answered by a diagram on its project and task written after it started; an older one cannot be its answer. */
function answers(summary: DiagramSummary, generation: PendingGeneration): boolean {
  const sameOrigin = summary.projectId === generation.projectId && summary.taskId === generation.taskId
  return sameOrigin && Date.parse(summary.updatedAt) >= generation.startedAt
}

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
    kinds: diagram.kinds,
    createdAt: diagram.createdAt,
    updatedAt: diagram.updatedAt
  }
}
