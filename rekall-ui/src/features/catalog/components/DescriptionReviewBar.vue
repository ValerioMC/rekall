<script setup lang="ts">
/**
 * The review line of a task with no checklist: what a session has done with it, and the answers
 * left to give. Sending back can carry a note, which the next session reads.
 */
import { ref, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppInput from '@/components/ui/AppInput.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import type { Task, TaskStepState } from '@/features/catalog/model/catalog'

const props = defineProps<{ task: Task; reviewState: Exclude<TaskStepState, 'OPEN'> }>()

const store = useConsoleStore()
const { run } = useAsyncAction()

const sendingBack = ref(false)
const sendBackNote = ref('')

watch(
  () => props.task.id,
  () => {
    sendingBack.value = false
    sendBackNote.value = ''
  }
)

async function acceptDescription(): Promise<void> {
  const id = props.task.id
  await run(() => store.acceptTask(id))
}

async function sendBackDescription(): Promise<void> {
  const id = props.task.id
  const note = sendBackNote.value.trim()
  sendingBack.value = false
  sendBackNote.value = ''
  await run(() => store.sendBackTask(id, note || undefined))
}

async function markTaskDone(): Promise<void> {
  const id = props.task.id
  await run(() => store.setTaskStatus(id, 'DONE'))
}
</script>

<template>
  <div
    class="shrink-0 border-b border-border bg-accent-soft/40 px-5 py-2.5"
    data-testid="description-review-bar"
  >
    <div v-if="!sendingBack" class="flex flex-wrap items-center gap-2">
      <span class="min-w-0 flex-1 text-caption leading-snug text-text-muted">
        <template v-if="reviewState === 'RUNNING'">
          A session is on this task. It moves to review on its own once the session writes the
          wrapup, and back to open when the session ends. Accept it now if you have seen enough.
        </template>
        <template v-else-if="reviewState === 'CLAIMED'">
          A session wrote the wrapup and claimed this. Accept it, or send it back to reopen it
          for another pass.
        </template>
        <template v-else>
          You accepted this. Send it back to reopen it, or mark the whole task done.
        </template>
      </span>
      <AppButton
        v-if="reviewState === 'CLAIMED' || reviewState === 'RUNNING'"
        variant="primary"
        size="sm"
        data-testid="description-accept"
        @click="acceptDescription"
      >
        Accept
      </AppButton>
      <AppButton
        variant="quiet"
        size="sm"
        data-testid="description-open-wrapup"
        @click="store.openWrapup()"
      >
        Open the wrapup
      </AppButton>
      <AppButton
        v-if="reviewState === 'DONE' && task.status !== 'DONE'"
        variant="primary"
        size="sm"
        data-testid="description-mark-done"
        @click="markTaskDone"
      >
        Mark task done
      </AppButton>
      <AppButton
        v-if="reviewState !== 'RUNNING'"
        variant="quiet"
        size="sm"
        data-testid="description-send-back"
        @click="sendingBack = true"
      >
        Send back
      </AppButton>
    </div>
    <div v-else class="flex flex-wrap items-center gap-2">
      <AppInput
        v-model="sendBackNote"
        class="min-w-0 flex-1"
        placeholder="Optional: what needs another pass. The next session sees this."
        aria-label="Send back note"
        data-testid="description-send-back-note"
      />
      <AppButton
        variant="danger"
        size="sm"
        data-testid="description-send-back-confirm"
        @click="sendBackDescription"
      >
        Send back
      </AppButton>
      <AppButton
        variant="quiet"
        size="sm"
        @click="sendingBack = false"
      >
        Cancel
      </AppButton>
    </div>
  </div>
</template>
