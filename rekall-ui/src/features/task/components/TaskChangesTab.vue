<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useRoute, useRouter } from 'vue-router'
import AppButton from '@/components/ui/AppButton.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import AppSkeleton from '@/components/ui/AppSkeleton.vue'
import SegmentedControl, { type SegmentOption } from '@/components/ui/SegmentedControl.vue'
import ChangeFileTree from '@/features/commits/components/ChangeFileTree.vue'
import ChangeScopeHeader from '@/features/commits/components/ChangeScopeHeader.vue'
import ChangeScopeTrack from '@/features/commits/components/ChangeScopeTrack.vue'
import ChangedFileSection from '@/features/commits/components/ChangedFileSection.vue'
import { useChangeReview } from '@/features/commits/composables/useChangeReview'
import { useTaskChanges } from '@/features/commits/composables/useTaskChanges'
import {
  changeSources,
  otherChanges,
  reviewFiles,
  scopeGroups,
  sourcesInScope,
  totalsOf,
  type ReviewFile,
  type StepRef
} from '@/features/commits/lib/change-sources'
import { ALL_CHANGES, sameScope, WORKING_TREE_SOURCE_ID, type ChangeScope, type DiffLayout } from '@/features/commits/model/changeScope'
import { useConsoleStore } from '@/stores/console.store'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskStepId } from '@/model/branded'

/**
 * The Changes tab: what the sessions changed in the application for this task, read as a review.
 * The track picks what to compare (all changes, a step, a commit, the working tree), the tree jumps
 * between files and the column is one continuous diff. Viewed is the reader's own mark.
 */
const props = defineProps<{ task: Task }>()

const LAYOUTS: readonly SegmentOption<DiffLayout>[] = [
  { value: 'split', label: 'Split' },
  { value: 'unified', label: 'Unified' }
]
const SCROLL_MARGIN = 12

const route = useRoute()
const router = useRouter()
const store = useConsoleStore()
const { selectedTaskSteps, selectedTaskCommitReferences } = storeToRefs(store)
const { changes, status, failure, reload } = useTaskChanges(
  () => props.task.id,
  () => selectedTaskCommitReferences.value.length
)
const review = useChangeReview()

const routedCommit = computed(() => (typeof route.query.commit === 'string' ? route.query.commit : null))
const scope = ref<ChangeScope>(routedCommit.value === null ? ALL_CHANGES : { kind: 'commit', referenceId: routedCommit.value })
const layout = ref<DiffLayout>('split')
const current = ref<string | null>(null)
const column = ref<HTMLElement | null>(null)

const steps = computed<readonly StepRef[]>(() =>
  selectedTaskSteps.value
    .filter((step) => step.state !== 'DRAFT')
    .map((step, index) => ({ id: step.id, number: index + 1, title: step.title, state: step.state }))
)
const stepNumbers = computed(() => new Map<TaskStepId, number>(steps.value.map((step) => [step.id, step.number])))
const stepTitles = computed(() => new Map<TaskStepId, string>(steps.value.map((step) => [step.id, step.title])))
const sources = computed(() => (changes.value === null ? [] : changeSources(changes.value)))
const groups = computed(() => scopeGroups(sources.value, steps.value))
const workingTree = computed(() => sources.value.find((source) => source.id === WORKING_TREE_SOURCE_ID) ?? null)
const allFiles = computed(() => reviewFiles(sources.value))
const scopedSources = computed(() => sourcesInScope(sources.value, scope.value))
const scopedFiles = computed(() => reviewFiles(scopedSources.value))
const scopedByPath = computed(() => new Map<string, ReviewFile>(scopedFiles.value.map((file) => [file.path, file])))
const totals = computed(() => totalsOf(scopedFiles.value))
const inContextIds = computed(
  () => new Set(selectedTaskCommitReferences.value.filter((reference) => reference.inContext).map((reference) => reference.id))
)
const scopedReference = computed(() => {
  if (scope.value.kind !== 'commit') return null
  const referenceId = scope.value.referenceId
  return selectedTaskCommitReferences.value.find((reference) => reference.id === referenceId) ?? null
})

watch(
  () => props.task.id,
  () => {
    review.reset()
    scope.value = ALL_CHANGES
    current.value = null
  }
)

watch(sources, (now) => {
  const stillThere = scope.value.kind === 'all' || scope.value.kind === 'step' || sourcesInScope(now, scope.value).length > 0
  if (!stillThere) scope.value = ALL_CHANGES
})

function scrollToTop(): void {
  column.value?.scrollTo({ top: 0 })
}

function selectScope(next: ChangeScope): void {
  if (sameScope(scope.value, next)) return
  scope.value = next
  current.value = null
  void nextTick(() => {
    current.value = scopedFiles.value[0]?.path ?? null
    scrollToTop()
  })
}

function syncCommitQuery(): void {
  const commit = scope.value.kind === 'commit' ? scope.value.referenceId : undefined
  if (commit === routedCommit.value || (commit === undefined && routedCommit.value === null)) return
  void router.replace({ query: { ...route.query, commit } })
}

watch(routedCommit, (commit) => {
  if (commit !== null) selectScope({ kind: 'commit', referenceId: commit })
})
watch(scope, syncCommitQuery)

function openSource(sourceId: string): void {
  selectScope(sourceId === WORKING_TREE_SOURCE_ID ? { kind: 'working-tree' } : { kind: 'commit', referenceId: sourceId })
}

function showFile(path: string): void {
  const reveal = (): void => {
    const target = column.value?.querySelector<HTMLElement>(`[data-file-path="${CSS.escape(path)}"]`)
    if (!target || !column.value) return
    current.value = path
    column.value.scrollTo({ top: target.offsetTop - SCROLL_MARGIN })
  }
  if (scopedByPath.value.has(path)) return reveal()
  scope.value = ALL_CHANGES
  void nextTick(reveal)
}

function stepFile(direction: 1 | -1): void {
  const paths = scopedFiles.value.map((file) => file.path)
  const at = current.value === null ? -1 : paths.indexOf(current.value)
  const next = paths[Math.min(Math.max(at + direction, 0), paths.length - 1)]
  if (next !== undefined) showFile(next)
}

function followScroll(): void {
  const element = column.value
  if (!element) return
  let reached: string | null = null
  for (const section of element.querySelectorAll<HTMLElement>('[data-file-path]')) {
    if (section.offsetTop - SCROLL_MARGIN * 6 <= element.scrollTop) reached = section.dataset.filePath ?? null
  }
  if (reached !== null) current.value = reached
}

const KEY_ACTIONS: Readonly<Record<string, () => void>> = {
  j: () => stepFile(1),
  k: () => stepFile(-1),
  u: () => (layout.value = layout.value === 'split' ? 'unified' : 'split'),
  a: () => selectScope(ALL_CHANGES),
  v: () => {
    if (current.value !== null) review.toggleViewed(current.value)
  }
}

function onKey(event: KeyboardEvent): void {
  if (event.metaKey || event.ctrlKey || event.altKey) return
  const target = event.target
  if (target instanceof HTMLElement && target.closest('input, textarea, select, [contenteditable="true"]')) return
  KEY_ACTIONS[event.key]?.()
}

onMounted(() => window.addEventListener('keydown', onKey))
onBeforeUnmount(() => window.removeEventListener('keydown', onKey))
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col" data-testid="task-changes-tab">
    <div v-if="status === 'loading'" class="px-6 py-5" data-testid="changes-loading"><AppSkeleton variant="list" /></div>

    <div v-else-if="status === 'failed'" class="grid flex-1 place-items-center px-6" data-testid="changes-failed">
      <div class="flex flex-col items-center gap-3 text-center">
        <p class="text-body font-medium text-text">The changes could not be read.</p>
        <p class="max-w-md text-small text-text-muted">{{ failure }}</p>
        <AppButton size="sm" @click="reload">Try again</AppButton>
      </div>
    </div>

    <div v-else-if="sources.length === 0" class="px-6 py-8" data-testid="changes-empty">
      <AppEmptyState title="Nothing has changed yet" description="Log a commit against this task, or let a session work on it: its commits and the files it is writing appear here.">
        <AppButton size="sm" @click="reload">Reload</AppButton>
      </AppEmptyState>
    </div>

    <template v-else>
      <div class="flex h-13 shrink-0 items-center gap-3 border-b border-border px-6">
        <ChangeScopeTrack :scope="scope" :groups="groups" :working-tree="workingTree" :in-context="inContextIds" class="min-w-0" @select="selectScope" />
        <span class="flex-1" />
        <p class="hidden items-center gap-3 text-caption text-text-subtle @5xl/task:flex">
          <span><kbd>j</kbd> <kbd>k</kbd> files</span>
          <span><kbd>v</kbd> viewed</span>
          <span><kbd>u</kbd> layout</span>
        </p>
        <AppButton variant="quiet" size="sm" data-testid="changes-reload" @click="reload">Reload</AppButton>
        <SegmentedControl v-model="layout" :options="LAYOUTS" label="Diff layout" size="sm" />
      </div>

      <div class="grid min-h-0 flex-1 grid-cols-[20.5rem_minmax(0,1fr)]">
        <ChangeFileTree :files="allFiles" :in-scope="scopedByPath" :viewed="review.viewed.value" :current="current" @select="showFile" />
        <div ref="column" class="relative min-h-0 overflow-auto bg-canvas" data-testid="changes-column" @scroll.passive="followScroll">
          <ChangeScopeHeader :scope="scope" :sources="scopedSources" :totals="totals" :step-titles="stepTitles" :step-numbers="stepNumbers" :reference="scopedReference" />
          <p v-if="scopedFiles.length === 0" class="px-6 py-8 text-small text-text-subtle" data-testid="changes-scope-empty">This change has no file to show.</p>
          <ChangedFileSection
            v-for="file in scopedFiles"
            :key="file.path"
            :file="file"
            :layout="layout"
            :scope="scope"
            :viewed="review.isViewed(file.path)"
            :shut="review.isShut(file.path)"
            :current="current === file.path"
            :other-changes="scope.kind === 'commit' || scope.kind === 'working-tree' ? otherChanges(file.path, sources, scopedSources) : []"
            :step-numbers="stepNumbers"
            @toggle-viewed="review.toggleViewed(file.path)"
            @toggle-shut="review.toggleShut(file.path)"
            @open-source="openSource"
          />
        </div>
      </div>
    </template>
  </div>
</template>
