<script setup lang="ts">
/**
 * Shown while a revision waits or a pass runs, whichever view of the description is open. Waiting,
 * it offers what the revision is applied to and the Run revision key; while a pass runs the key
 * turns into Working and a revision written meanwhile waits for the next run. The change itself is
 * read in the Revisions tab.
 */
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import ApplyFeedbackButton from '@/features/feedback/components/ApplyFeedbackButton.vue'
import RevisionTargetSwitch from '@/features/feedback/components/RevisionTargetSwitch.vue'
import { useRevisionRun } from '@/features/feedback/composables/useRevisionRun'
import { isWaiting, type FeedbackKind } from '@/features/feedback/model/feedback'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import type { TaskId } from '@/model/branded'

const props = defineProps<{ taskId: TaskId; anchor: string; folder: string | null; revisionsShown: boolean }>()
const emit = defineEmits<{ showRevisions: [] }>()

const store = useConsoleStore()
const { selectedTaskFeedback } = storeToRefs(store)
const { run } = useAsyncAction()
const { phase, visible, hint, waitingCount, runningPass, apply } = useRevisionRun(
  () => props.taskId,
  () => props.anchor,
  () => props.folder
)

const waiting = computed(() => selectedTaskFeedback.value.filter(isWaiting))
const target = computed<FeedbackKind>(() => waiting.value[0]?.kind ?? 'IMPLEMENTATION')
const waitingSummary = computed(() => (waitingCount.value === 1 ? '1 revision waiting' : `${waitingCount.value} revisions waiting`))
const summary = computed(() => {
  if (runningPass.value !== null) {
    return waitingCount.value === 0 ? `Pass ${runningPass.value} is running` : `Pass ${runningPass.value} is running, ${waitingSummary.value}`
  }
  if (phase.value === 'launching' || phase.value === 'waiting') return 'Starting a session'
  if (phase.value === 'done') return 'Revision finished'
  return waitingSummary.value
})

function aim(kind: FeedbackKind): void {
  void run(async () => {
    for (const entry of waiting.value.filter((candidate) => candidate.kind !== kind)) await store.retargetFeedback(entry, kind)
  })
}
</script>

<template>
  <div v-if="visible" class="flex shrink-0 flex-wrap items-center gap-2 border-b border-border bg-canvas px-5 py-2" data-testid="revision-run-bar">
    <span class="text-small font-medium text-text" data-testid="revision-run-summary">{{ summary }}</span>
    <RevisionTargetSwitch v-if="phase === 'idle'" :model-value="target" @update:model-value="aim" />
    <span class="flex-1" />
    <AppButton v-if="!revisionsShown" size="sm" variant="quiet" data-testid="revision-run-view" @click="emit('showRevisions')">
      See the changes
    </AppButton>
    <ApplyFeedbackButton :phase="phase" :count="waitingCount" :hint="hint" @press="apply" />
  </div>
</template>
