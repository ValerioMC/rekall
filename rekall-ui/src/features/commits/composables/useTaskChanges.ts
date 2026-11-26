import { computed, onScopeDispose, ref, shallowReactive, shallowRef, watch, type Ref } from 'vue'
import { ApiError } from '@/api/client'
import { fetchTaskChangeFiles, fetchTaskChangesIndex } from '@/features/commits/api/taskChanges.api'
import {
  attachFileHunks,
  changeSources,
  reviewFiles,
  sourcesInScope,
  type ChangeSource,
  type ReviewFile,
  type ReviewFileDiff
} from '@/features/commits/lib/change-sources'
import { ALL_CHANGES, PROJECT_REPOSITORY_ID, sameScope, type ChangeScope } from '@/features/commits/model/changeScope'
import type { TaskChangesIndex } from '@/features/commits/model/taskChanges'
import type { TaskId } from '@/model/branded'

type TaskChangesStatus = 'loading' | 'ready' | 'failed'

type BatchState =
  | { kind: 'pending' }
  | { kind: 'failed'; message: string }
  | { kind: 'ready'; files: readonly ReviewFileDiff[] }

interface ChangesRead {
  readonly task: TaskId
  readonly snapshotId: string
  readonly sources: readonly ChangeSource[]
}

interface ScopedBatches {
  readonly read: ChangesRead
  readonly scope: ChangeScope
  readonly files: readonly ReviewFile[]
  readonly batches: Map<number, BatchState>
  recovery?: Promise<ScopedBatches | null>
}

const LOADING_DELAY = 200
const FILE_BATCH_SIZE = 8

function createRead(task: TaskId, index: TaskChangesIndex): ChangesRead {
  return { task, snapshotId: index.snapshotId, sources: changeSources(index) }
}

function createScopedBatches(read: ChangesRead, scope: ChangeScope): ScopedBatches {
  return {
    read,
    scope,
    files: reviewFiles(sourcesInScope(read.sources, scope)),
    batches: shallowReactive(new Map<number, BatchState>())
  }
}

function availableScope(read: ChangesRead, scope: ChangeScope): ChangeScope {
  if (scope.kind === 'step' || sourcesInScope(read.sources, scope).length > 0) return scope
  return ALL_CHANGES
}

function filesAt(active: ScopedBatches, start: number): readonly ReviewFile[] {
  return active.files.slice(start, start + FILE_BATCH_SIZE)
}

function batchStart(index: number): number {
  return index - index % FILE_BATCH_SIZE
}

function batchStartsForFiles(active: ScopedBatches, fileIds: ReadonlySet<string>): number[] {
  return [...new Set(active.files.flatMap((file, index) => fileIds.has(file.id) ? [batchStart(index)] : []))]
}

async function readBatch(active: ScopedBatches, start: number): Promise<readonly ReviewFileDiff[]> {
  const files = filesAt(active, start)
  const keys = files.map((file) => ({
    componentId: file.repository.id === PROJECT_REPOSITORY_ID ? null : file.repository.id,
    path: file.path
  }))
  const entries = await fetchTaskChangeFiles(active.read.task, active.read.snapshotId, active.scope, keys)
  return attachFileHunks(files, entries)
}

function failureMessage(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback
}

export function useTaskChanges(taskId: () => TaskId, refreshKey: () => unknown, scope: Ref<ChangeScope>) {
  const current = shallowRef<ScopedBatches | null>(null)
  const status = ref<TaskChangesStatus>('loading')
  const failure = ref<string | null>(null)
  const showLoading = ref(false)
  let readVersion = 0
  let loadingTimer: ReturnType<typeof setTimeout> | undefined

  const sources = computed(() => current.value?.read.sources ?? [])
  const scopedFiles = computed(() => current.value?.files ?? [])
  const batches = computed(() => {
    const active = current.value
    if (!active) return []
    return Array.from({ length: Math.ceil(active.files.length / FILE_BATCH_SIZE) }, (_, index) => {
      const start = index * FILE_BATCH_SIZE
      return {
        start,
        size: Math.min(FILE_BATCH_SIZE, active.files.length - start),
        state: active.batches.get(start),
        continuesPrevious: active.batches.get(start - FILE_BATCH_SIZE)?.kind === 'ready'
      }
    })
  })
  const trailingBatch = computed(() => {
    const lastReady = batches.value.reduce((last, batch, index) => batch.state?.kind === 'ready' ? index : last, -1)
    return batches.value[lastReady + 1]
  })
  const visibleBatches = computed(() => batches.value.filter((batch, index, ordered) =>
    batch.state || index === 0 || ordered[index - 1]?.state?.kind === 'ready'
  ))

  function isCurrent(active: ScopedBatches): boolean {
    return current.value === active && active.read.task === taskId() && sameScope(active.scope, scope.value)
  }

  function clearLoadingDelay(): void {
    clearTimeout(loadingTimer)
    loadingTimer = undefined
    showLoading.value = false
  }

  function publishScope(active: ScopedBatches): void {
    scope.value = active.scope
    current.value = active
    requestBatch(0)
  }

  async function reload(): Promise<void> {
    const task = taskId()
    const version = ++readVersion
    clearLoadingDelay()
    current.value = null
    status.value = 'loading'
    loadingTimer = setTimeout(() => (showLoading.value = true), LOADING_DELAY)
    try {
      const index = await fetchTaskChangesIndex(task)
      if (version !== readVersion || task !== taskId()) return
      const read = createRead(task, index)
      publishScope(createScopedBatches(read, availableScope(read, scope.value)))
      failure.value = null
      status.value = 'ready'
    } catch (error) {
      if (version !== readVersion || task !== taskId()) return
      failure.value = failureMessage(error, 'The changes could not be read.')
      status.value = 'failed'
    } finally {
      if (version === readVersion) clearLoadingDelay()
    }
  }

  async function refreshExpiredSnapshot(active: ScopedBatches): Promise<ScopedBatches | null> {
    const index = await fetchTaskChangesIndex(active.read.task)
    if (!isCurrent(active)) return null
    const read = createRead(active.read.task, index)
    const replacement = createScopedBatches(read, availableScope(read, active.scope))
    const requestedIds = new Set([...active.batches.keys()].flatMap((start) => filesAt(active, start).map((file) => file.id)))
    const starts = batchStartsForFiles(replacement, requestedIds)
    if (starts.length === 0 && replacement.files.length > 0) starts.push(0)
    await Promise.all(starts.map(async (start) => {
      const files = await readBatch(replacement, start)
      replacement.batches.set(start, { kind: 'ready', files })
    }))
    if (!isCurrent(active)) return null
    scope.value = replacement.scope
    current.value = replacement
    return replacement
  }

  async function recoverBatch(active: ScopedBatches, start: number): Promise<void> {
    active.recovery ??= refreshExpiredSnapshot(active).finally(() => {
      active.recovery = undefined
    })
    const replacement = await active.recovery
    if (!replacement || !isCurrent(replacement)) return
    const wanted = new Set(filesAt(active, start).map((file) => file.id))
    for (const needed of batchStartsForFiles(replacement, wanted)) requestBatch(needed)
  }

  async function fillBatch(active: ScopedBatches, start: number): Promise<void> {
    if (active.recovery) {
      await recoverBatch(active, start)
      return
    }
    try {
      const files = await readBatch(active, start)
      if (isCurrent(active)) active.batches.set(start, { kind: 'ready', files })
    } catch (error) {
      if (!isCurrent(active)) return
      if (!(error instanceof ApiError) || error.status !== 409) throw error
      await recoverBatch(active, start)
    }
  }

  async function loadBatch(active: ScopedBatches, start: number): Promise<void> {
    active.batches.set(start, { kind: 'pending' })
    try {
      await fillBatch(active, start)
    } catch (error) {
      if (isCurrent(active)) {
        active.batches.set(start, { kind: 'failed', message: failureMessage(error, 'The file diffs could not be read.') })
      }
    }
  }

  function requestBatch(start: number): void {
    const active = current.value
    if (!active || !isCurrent(active) || start >= active.files.length) return
    const state = active.batches.get(start)
    if (state?.kind === 'pending' || state?.kind === 'ready') return
    void loadBatch(active, start)
  }

  function requestFile(fileId: string): void {
    const index = scopedFiles.value.findIndex((file) => file.id === fileId)
    if (index >= 0) requestBatch(batchStart(index))
  }

  watch(scope, (wanted) => {
    const active = current.value
    if (active && !sameScope(active.scope, wanted)) {
      publishScope(createScopedBatches(active.read, wanted))
    }
  })
  watch([taskId, refreshKey], () => void reload(), { immediate: true })
  onScopeDispose(() => {
    readVersion++
    current.value = null
    clearLoadingDelay()
  })

  return { sources, scopedFiles, status, failure, showLoading, reload, trailingBatch, visibleBatches, requestBatch, requestFile }
}
