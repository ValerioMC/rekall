<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'
import ChatEnded from './ChatEnded.vue'
import ChatMessage from './ChatMessage.vue'
import ChatNotice from './ChatNotice.vue'
import ChatThinking from './ChatThinking.vue'
import ChatTurnFooter from './ChatTurnFooter.vue'
import ToolCallCard from './ToolCallCard.vue'
import { useStickToBottom } from '@/features/chat/composables/useStickToBottom'
import type { TimelineItem } from '@/features/chat/model/timeline'

/**
 * The conversation as a scrolling log. Only the last 150 items are mounted, and earlier ones
 * come back 150 at a time. It follows new content while the reader is at the bottom and lets go
 * the moment they scroll up.
 */
const props = withDefaults(defineProps<{ items: readonly TimelineItem[]; followingNext?: boolean }>(), {
  followingNext: false
})

const WINDOW_ITEMS = 150

const scroller = ref<HTMLElement | null>(null)
const mounted = ref(WINDOW_ITEMS)

const { stuck, unseen, onScroll, jumpToLatest } = useStickToBottom(scroller, () => props.items)

const hidden = computed(() => Math.max(0, props.items.length - mounted.value))
const visible = computed(() => props.items.slice(Math.max(0, props.items.length - mounted.value)))
const earlierCount = computed(() => Math.min(WINDOW_ITEMS, hidden.value))

async function showEarlier(): Promise<void> {
  const element = scroller.value
  const heightBefore = element?.scrollHeight ?? 0
  mounted.value += WINDOW_ITEMS
  await nextTick()
  if (element) element.scrollTop += element.scrollHeight - heightBefore
}
</script>

<template>
  <div class="relative min-h-0 flex-1">
    <div
      ref="scroller"
      class="focus-ring h-full overflow-y-auto px-4 py-3"
      role="log"
      aria-label="Conversation"
      aria-live="off"
      tabindex="0"
      data-testid="chat-timeline"
      @scroll.passive="onScroll"
    >
      <ol class="flex flex-col gap-3">
        <li v-if="hidden > 0" class="flex justify-center">
          <button
            type="button"
            class="focus-ring key-quiet h-7 cursor-pointer rounded-key px-3 text-caption"
            data-testid="chat-show-earlier"
            @click="showEarlier"
          >
            Show {{ earlierCount }} earlier {{ earlierCount === 1 ? 'item' : 'items' }}
          </button>
        </li>
        <li v-for="item in visible" :key="item.key">
          <ChatMessage v-if="item.kind === 'user'" role="user" :text="item.text" :complete="true" />
          <ChatMessage v-else-if="item.kind === 'assistant'" role="assistant" :text="item.text" :complete="item.complete" />
          <ChatThinking v-else-if="item.kind === 'thinking'" :text="item.text" :complete="item.complete" />
          <ToolCallCard v-else-if="item.kind === 'tool'" :call="item" />
          <ChatNotice v-else-if="item.kind === 'notice'" :kind="item.noticeKind" :text="item.text" />
          <ChatTurnFooter
            v-else-if="item.kind === 'turn-footer'"
            :usage="item.usage"
            :duration-ms="item.durationMs"
            :outcome="item.outcome"
          />
          <ChatEnded v-else :exit-code="item.exitCode" :detail="item.detail" :following-next="followingNext" />
        </li>
      </ol>
    </div>

    <button
      v-if="!stuck"
      type="button"
      class="focus-ring key-gold absolute bottom-3 left-1/2 h-7 -translate-x-1/2 cursor-pointer rounded-key px-3 text-caption font-semibold"
      data-testid="chat-jump-to-latest"
      @click="jumpToLatest"
    >
      Jump to latest<template v-if="unseen > 0"> · {{ unseen }} new</template>
    </button>
  </div>
</template>
