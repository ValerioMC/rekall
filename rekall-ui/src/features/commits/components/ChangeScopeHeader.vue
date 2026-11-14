<script setup lang="ts">
import { computed } from 'vue'
import { relativeTime } from '@/common/format/relative-time'
import type { ChangeSource, ChangeTotals } from '@/features/commits/lib/change-sources'
import type { ChangeScope } from '@/features/commits/model/changeScope'
import type { TaskStepId } from '@/model/branded'

/** What the diff below is: the net result, a step, one commit or the working tree, with its totals. */
const props = defineProps<{
  scope: ChangeScope
  sources: readonly ChangeSource[]
  totals: ChangeTotals
  stepTitles: ReadonlyMap<TaskStepId, string>
  stepNumbers: ReadonlyMap<TaskStepId, number>
}>()

const only = computed(() => props.sources[0] ?? null)
const unreadable = computed(() => props.sources.filter((source) => !source.available))
const truncated = computed(() => props.sources.filter((source) => source.truncated))

const heading = computed(() => {
  if (props.scope.kind === 'all') return 'Everything the sessions changed for this task'
  if (props.scope.kind === 'step') return props.stepTitles.get(props.scope.stepId) ?? 'A step'
  if (props.scope.kind === 'working-tree') return 'Working tree'
  return only.value?.subject ?? 'A commit'
})
const eyebrow = computed(() => {
  if (props.scope.kind === 'all') return 'Changes logged for this task'
  if (props.scope.kind === 'step') return `Step ${props.stepNumbers.get(props.scope.stepId) ?? ''}`
  if (props.scope.kind === 'working-tree') return 'Not committed'
  return only.value?.commitHash?.slice(0, 7) ?? ''
})
const explanation = computed(() => {
  if (props.scope.kind === 'all') return 'Each file as the logged commits left it. Every hunk names the commit that made it: open one to read that change on its own.'
  if (props.scope.kind === 'step') return 'The commits logged against this step, together.'
  if (props.scope.kind === 'working-tree') return 'What a session has written and not committed yet. It can still change; reload to read it again.'
  return only.value?.loggedAt ? `Logged ${relativeTime(only.value.loggedAt)}.` : ''
})
</script>

<template>
  <header class="border-b border-border bg-surface px-6 pb-4 pt-4.5" data-testid="change-scope-header">
    <p class="flex items-center gap-2 text-caption text-text-subtle">
      <span v-if="scope.kind === 'commit'" class="rounded-control border border-border-strong bg-canvas px-1.5 py-px font-mono text-micro text-text-muted">{{ eyebrow }}</span>
      <template v-else>{{ eyebrow }}</template>
    </p>
    <h2 class="mt-1.5 text-title font-semibold tracking-[-0.01em] text-text">{{ heading }}</h2>
    <p class="mt-1.5 max-w-prose text-small text-text-muted">{{ explanation }}</p>
    <p class="mt-3 flex flex-wrap items-center gap-4 text-small text-text-muted" data-testid="change-totals">
      <span><b class="font-medium text-text">{{ totals.files }}</b> {{ totals.files === 1 ? 'file' : 'files' }}</span>
      <span class="font-mono tabular-nums"><span class="text-safe">+{{ totals.additions }}</span> <span class="text-danger">−{{ totals.deletions }}</span></span>
      <span v-if="totals.commits > 0"><b class="font-medium text-text">{{ totals.commits }}</b> {{ totals.commits === 1 ? 'commit' : 'commits' }}</span>
    </p>
    <p v-for="source in unreadable" :key="`unreadable-${source.id}`" class="mt-2 text-caption text-warn" data-testid="change-unreadable">
      No patch could be read for {{ source.commitHash?.slice(0, 7) }}: git no longer has it and no diff was stored.
    </p>
    <p v-for="source in truncated" :key="`truncated-${source.id}`" class="mt-2 text-caption text-warn" data-testid="change-truncated">
      The patch of {{ source.commitHash?.slice(0, 7) ?? 'the working tree' }} is cut where it grew too large.
    </p>
  </header>
</template>
