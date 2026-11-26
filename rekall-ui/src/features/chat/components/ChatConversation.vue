<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import ChatApprovalCard from './ChatApprovalCard.vue'
import ChatComposer from './ChatComposer.vue'
import ChatTimeline from './ChatTimeline.vue'
import { composerState, type ChatStatus } from '@/features/chat/lib/chat-status'
import { failureMessage } from '@/features/chat/lib/failure-message'
import { useChatStore } from '@/features/chat/store/chat.store'
import type { Terminal } from '@/features/claude/model/terminal'

/**
 * One chat's conversation and the answers it waits for: messages, stop, and the allow or deny of a
 * tool call. Under a container named `talk` it reads as a centred column and keeps clear of the dock.
 */
const props = defineProps<{ session: Terminal; status: ChatStatus }>()

const chat = useChatStore()

const composer = ref<InstanceType<typeof ChatComposer> | null>(null)
const sendError = ref<string | null>(null)
const answerError = ref<string | null>(null)
const answeredRequestId = ref<string | null>(null)
const stopRequested = ref(false)

const timeline = computed(() => chat.timelineOf(props.session.id))
const pending = computed(() => timeline.value.pending)
const followingNext = computed(() => chat.isFollowing(props.session.id))
const answering = computed(
  () => chat.answering || (pending.value !== null && answeredRequestId.value === pending.value.requestId)
)

async function send(text: string): Promise<void> {
  sendError.value = null
  try {
    await chat.send(props.session.id, text)
    composer.value?.clear()
  } catch (error) {
    sendError.value = failureMessage(error)
  }
}

async function stopTurn(): Promise<void> {
  sendError.value = null
  stopRequested.value = true
  try {
    await chat.stop(props.session.id)
  } catch (error) {
    stopRequested.value = false
    sendError.value = failureMessage(error)
  }
}

async function answer(allowed: boolean): Promise<void> {
  const request = pending.value
  if (!request) return
  answerError.value = null
  try {
    if (allowed) await chat.allow(props.session.id)
    else await chat.deny(props.session.id)
    answeredRequestId.value = request.requestId
  } catch (error) {
    answerError.value = failureMessage(error)
  }
}

watch(
  () => props.session.id,
  () => {
    sendError.value = null
    answerError.value = null
    stopRequested.value = false
  }
)

watch(
  () => timeline.value.turn,
  (turn) => {
    if (turn !== 'working') stopRequested.value = false
  }
)

watch(pending, async (request, previous) => {
  answerError.value = null
  if (request || !previous) return
  await nextTick()
  composer.value?.focus()
})

onMounted(async () => {
  await nextTick()
  if (!pending.value) composer.value?.focus()
})
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col">
    <p
      v-if="status === 'reconnecting'"
      class="shrink-0 border-b border-warn/40 bg-warn-soft px-4 py-1.5 text-caption text-warn"
      role="status"
      data-testid="chat-reconnecting"
    >
      Reconnecting…
    </p>

    <ChatTimeline :items="timeline.items" :following-next="followingNext" />

    <div class="shrink-0 border-t border-border">
      <div class="mx-auto w-full max-w-[748px] px-4 pt-3 pb-2 @min-[560px]/talk:px-6 @max-[1100px]/talk:pb-3">
        <ChatApprovalCard
          v-if="pending"
          :key="pending.requestId"
          class="mb-3"
          :request="pending"
          :provider="session.provider"
          :busy="answering"
          :error="answerError"
          @allow="answer(true)"
          @deny="answer(false)"
        />

        <ChatComposer
          ref="composer"
          :provider="session.provider"
          :state="composerState(status)"
          :sending="chat.sending"
          :stopping="stopRequested"
          :error="sendError"
          :following-next="followingNext"
          @send="send"
          @stop="stopTurn"
        />
      </div>
    </div>
  </div>
</template>
