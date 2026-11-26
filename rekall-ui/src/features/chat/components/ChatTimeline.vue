<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'
import ChatTimelineRows from './ChatTimelineRows.vue'
import { groupByParent } from '@/features/chat/lib/group-by-parent'
import { useStickToBottom } from '@/features/chat/composables/useStickToBottom'
import type { TimelineItem } from '@/features/chat/model/timeline'

/**
 * The conversation as a scrolling log. Only the last 150 items of the conversation are mounted,
 * and earlier ones come back 150 at a time; a subagent's items are drawn inside its Agent card and
 * do not count. It follows new content while the reader is at the bottom and lets go the moment
 * they scroll up.
 */
const props = withDefaults(defineProps<{ items: readonly TimelineItem[]; followingNext?: boolean }>(), {
  followingNext: false
})

const WINDOW_ITEMS = 150

const scroller = ref<HTMLElement | null>(null)
const mounted = ref(WINDOW_ITEMS)

const { stuck, unseen, onScroll, jumpToLatest } = useStickToBottom(scroller, () => props.items)

const grouped = computed(() => groupByParent(props.items))
const hidden = computed(() => Math.max(0, grouped.value.top.length - mounted.value))
const visible = computed(() => grouped.value.top.slice(hidden.value))
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
      class="focus-ring h-full overflow-y-auto py-3 @min-[560px]/talk:py-5 @min-[560px]/talk:[scrollbar-gutter:stable_both-edges]"
      role="log"
      aria-label="Conversation"
      aria-live="off"
      tabindex="0"
      data-testid="chat-timeline"
      @scroll.passive="onScroll"
    >
      <ol class="mx-auto flex w-full max-w-[748px] flex-col gap-3 px-4 @min-[560px]/talk:px-6">
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
        <ChatTimelineRows :items="visible" :inner="grouped.inner" :following-next="followingNext" />
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
