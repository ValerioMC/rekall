<script setup lang="ts">
/**
 * Shown while a revision waits, whichever view of the description is open: what it is applied to,
 * chosen here, and the one Run revision button. The change itself is read in the Revisions tab.
 */
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import ApplyFeedbackButton from '@/features/feedback/components/ApplyFeedbackButton.vue'
import RevisionTargetSwitch from '@/features/feedback/components/RevisionTargetSwitch.vue'
import { isWaiting, type FeedbackKind } from '@/features/feedback/model/feedback'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import type { TaskId } from '@/model/branded'

defineProps<{ taskId: TaskId; anchor: string; folder: string | null; revisionsShown: boolean }>()
const emit = defineEmits<{ showRevisions: [] }>()

const store = useConsoleStore()
const { selectedTaskFeedback, waitingCount } = storeToRefs(store)
const { run } = useAsyncAction()

const waiting = computed(() => selectedTaskFeedback.value.filter(isWaiting))
const target = computed<FeedbackKind>(() => waiting.value[0]?.kind ?? 'IMPLEMENTATION')
const summary = computed(() => (waitingCount.value === 1 ? '1 revision waiting' : `${waitingCount.value} revisions waiting`))

function aim(kind: FeedbackKind): void {
  void run(async () => {
    for (const entry of waiting.value.filter((candidate) => candidate.kind !== kind)) await store.retargetFeedback(entry, kind)
  })
}
</script>

<template>
  <div class="flex shrink-0 flex-wrap items-center gap-2 border-b border-border bg-surface px-5 py-2" data-testid="revision-run-bar">
    <span class="text-small font-medium text-text" data-testid="revision-run-summary">{{ summary }}</span>
    <RevisionTargetSwitch :model-value="target" @update:model-value="aim" />
    <span class="flex-1" />
    <AppButton v-if="!revisionsShown" size="sm" variant="quiet" data-testid="revision-run-view" @click="emit('showRevisions')">
      See the changes
    </AppButton>
    <ApplyFeedbackButton variant="primary" :task-id="taskId" :anchor="anchor" :folder="folder" />
  </div>
</template>
