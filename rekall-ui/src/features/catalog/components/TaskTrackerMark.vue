<script setup lang="ts">
import { computed } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useExternalLink } from '@/composables/useExternalLink'
import TrackerSyncGlyph, { type TrackerSyncState } from '@/features/catalog/components/TrackerSyncGlyph.vue'
import type { Task } from '@/features/catalog/model/catalog'
import { useConsoleStore } from '@/stores/console.store'

/**
 * The issue the project's task manager holds for the task, as a link to it, and the key that writes
 * the task's title and description there. The issue is created with the title alone and nothing
 * rewrites it on its own: the key shows a dot once the task has changed since, spins while it
 * writes, turns into a check when the issue is current and into a warning, with the reason, when
 * the write failed.
 */
const props = defineProps<{
  task: Pick<Task, 'id' | 'trackerIssueKey' | 'trackerIssueUrl' | 'trackerSyncError' | 'trackerContentSynced'>
}>()

const { follow } = useExternalLink()
const store = useConsoleStore()
const { isRunning: writing, run } = useAsyncAction()

const state = computed<TrackerSyncState>(() => {
  if (writing.value) return 'syncing'
  if (props.task.trackerSyncError) return 'failed'
  return props.task.trackerContentSynced ? 'synced' : 'pending'
})

const target = computed(() => (props.task.trackerIssueKey ? `issue ${props.task.trackerIssueKey}` : 'the task manager'))

const action = computed(() => {
  if (!props.task.trackerIssueKey) return 'Create the issue in the task manager'
  return `Write the title and description to ${target.value}`
})

const hint = computed(() => {
  switch (state.value) {
    case 'syncing':
      return 'Writing to the task manager'
    case 'synced':
      return `Synced with ${target.value}`
    case 'failed':
      return `${props.task.trackerSyncError ?? ''} Press to try again.`
    default:
      return props.task.trackerIssueKey ? `Changed since the last sync. ${action.value}` : action.value
  }
})

const TONES: Readonly<Record<TrackerSyncState, string>> = {
  pending: 'cursor-pointer text-accent hover:bg-accent-soft',
  syncing: 'cursor-progress text-accent',
  synced: 'cursor-default text-safe',
  failed: 'cursor-pointer text-warn hover:bg-warn-soft'
}

async function sync(): Promise<void> {
  if (state.value === 'syncing' || state.value === 'synced') return
  await run(() => store.syncTaskTracker(props.task.id))
}
</script>

<template>
  <span v-if="task.trackerIssueKey || task.trackerSyncError" class="inline-flex shrink-0 items-center gap-1">
    <a
      v-if="task.trackerIssueKey && task.trackerIssueUrl"
      :href="task.trackerIssueUrl"
      target="_blank"
      rel="noopener noreferrer"
      class="focus-ring rounded-control"
      :aria-label="`Open issue ${task.trackerIssueKey} in the task manager`"
      data-testid="tracker-issue-link"
      @click="follow($event, task.trackerIssueUrl)"
    >
      <AppBadge tone="accent" mono>{{ task.trackerIssueKey }} &#8599;</AppBadge>
    </a>
    <button
      type="button"
      class="focus-ring grid size-6 shrink-0 place-items-center rounded-control transition-colors duration-(--motion-quick)"
      :class="TONES[state]"
      :title="hint"
      :aria-label="action"
      :aria-disabled="state === 'syncing' || state === 'synced'"
      :aria-busy="state === 'syncing'"
      :data-state="state"
      data-testid="tracker-sync"
      @click="sync"
    >
      <TrackerSyncGlyph :state="state" />
    </button>
    <AppBadge v-if="task.trackerSyncError" tone="warn" :title="task.trackerSyncError" data-testid="tracker-sync-error">
      Not synced
    </AppBadge>
  </span>
</template>
