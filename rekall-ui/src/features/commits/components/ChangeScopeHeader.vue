<script setup lang="ts">
import { computed } from 'vue'
import { relativeTime } from '@/common/format/relative-time'
import CommitContextToggle from '@/features/commits/components/CommitContextToggle.vue'
import type { CommitReference } from '@/features/commits/model/commitReference'
import type { ChangeSource, ChangeTotals } from '@/features/commits/lib/change-sources'
import type { ChangeScope } from '@/features/commits/model/changeScope'
import type { TaskStepId } from '@/model/branded'

/** What the diff below is: the net result, a repository, a step, one commit or a working tree, with its totals. */
const props = defineProps<{
  scope: ChangeScope
  sources: readonly ChangeSource[]
  totals: ChangeTotals
  /** How many repositories the task changed: above one, the header says which one a commit or working tree is in. */
  repositoryCount: number
  stepTitles: ReadonlyMap<TaskStepId, string>
  stepNumbers: ReadonlyMap<TaskStepId, number>
  /** The logged commit when the scope is one commit: the one a person can hand to `/rk`. */
  reference: CommitReference | null
}>()

const only = computed(() => props.sources[0] ?? null)
const unreadable = computed(() => props.sources.filter((source) => !source.available))
const truncated = computed(() => props.sources.filter((source) => source.truncated))

const several = computed(() => props.repositoryCount > 1)
const repositoryName = computed(() => only.value?.repository.name ?? '')
const repositoriesInScope = computed(() => new Set(props.sources.map((source) => source.repository.id)).size)

const heading = computed(() => {
  if (props.scope.kind === 'all') return 'Everything the sessions changed for this task'
  if (props.scope.kind === 'repository') return repositoryName.value
  if (props.scope.kind === 'step') return props.stepTitles.get(props.scope.stepId) ?? 'A step'
  if (props.scope.kind === 'working-tree') return 'Working tree'
  return only.value?.subject ?? 'A commit'
})
const eyebrow = computed(() => {
  if (props.scope.kind === 'all') return 'Changes logged for this task'
  if (props.scope.kind === 'repository') return 'Repository'
  if (props.scope.kind === 'step') return `Step ${props.stepNumbers.get(props.scope.stepId) ?? ''}`
  if (props.scope.kind === 'working-tree') return 'Not committed'
  return only.value?.commitHash?.slice(0, 7) ?? ''
})
const explanation = computed(() => {
  if (props.scope.kind === 'all') {
    const grouped = several.value ? ' Files are grouped by repository.' : ''
    return `Each file as the logged commits left it. Every hunk names the commit that made it: open one to read that change on its own.${grouped}`
  }
  if (props.scope.kind === 'repository')
    return 'Everything the sessions changed in this repository, as its logged commits and working tree left it.'
  if (props.scope.kind === 'step') return 'The commits logged against this step, together.'
  if (props.scope.kind === 'working-tree') {
    const where = several.value ? ` in ${repositoryName.value}` : ''
    return `What a session has written${where} and not committed yet. It can still change; reload to read it again.`
  }
  if (!only.value?.loggedAt) return ''
  return `Logged ${relativeTime(only.value.loggedAt)}${several.value ? ` in ${repositoryName.value}` : ''}.`
})
</script>

<template>
  <header class="border-b border-border bg-surface px-6 pb-4 pt-4.5" data-testid="change-scope-header">
    <div class="flex items-start justify-between gap-6">
      <div class="min-w-0">
        <p class="flex items-center gap-2 text-caption text-text-subtle">
          <span v-if="scope.kind === 'commit'" class="rounded-control border border-border-strong bg-canvas px-1.5 py-px font-mono text-micro text-text-muted">{{ eyebrow }}</span>
          <template v-else>{{ eyebrow }}</template>
        </p>
        <h2 class="mt-1.5 text-title font-semibold tracking-[-0.01em] text-text">{{ heading }}</h2>
        <p class="mt-1.5 max-w-prose text-small text-text-muted">{{ explanation }}</p>
      </div>
      <div v-if="reference" class="flex shrink-0 flex-col items-end gap-1.5" data-testid="change-context-control">
        <CommitContextToggle :reference="reference" variant="labelled" />
        <p class="max-w-56 text-right text-caption text-text-subtle">
          {{ reference.inContext ? 'Its hash and diff arrive with /rk for this task.' : 'Hand its hash and diff to /rk for this task.' }}
        </p>
      </div>
    </div>
    <p class="mt-3 flex flex-wrap items-center gap-4 text-small text-text-muted" data-testid="change-totals">
      <span
        ><b class="font-medium text-text">{{ totals.files }}</b>
        {{ totals.files === 1 ? 'file' : 'files' }}</span
      >
      <span class="font-mono tabular-nums"
        ><span class="text-safe">+{{ totals.additions }}</span>
        <span class="text-danger">−{{ totals.deletions }}</span></span
      >
      <span v-if="scope.kind === 'all' && repositoriesInScope > 1"
        ><b class="font-medium text-text">{{ repositoriesInScope }}</b> repositories</span
      >
      <span v-if="totals.commits > 0"
        ><b class="font-medium text-text">{{ totals.commits }}</b>
        {{ totals.commits === 1 ? 'commit' : 'commits' }}</span
      >
    </p>
    <p v-for="source in unreadable" :key="`unreadable-${source.id}`" class="mt-2 text-caption text-warn" data-testid="change-unreadable">
      No patch could be read for {{ source.commitHash?.slice(0, 7) }}: git no longer has it and no diff was stored.
    </p>
    <p v-for="source in truncated" :key="`truncated-${source.id}`" class="mt-2 text-caption text-warn" data-testid="change-truncated">
      The patch of {{ source.commitHash?.slice(0, 7) ?? 'the working tree' }} is cut where it grew too large.
    </p>
  </header>
</template>
