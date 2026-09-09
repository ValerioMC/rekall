<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import ClaudeMessageBubble from '@/components/claude/ClaudeMessageBubble.vue'
import ClaudeToolCall from '@/components/claude/ClaudeToolCall.vue'
import { parseToolMeta, type ClaudeMessage } from '@/model/claude'

const props = defineProps<{ messages: readonly ClaudeMessage[]; working: boolean }>()

interface MessageRow {
  readonly kind: 'message'
  readonly key: string
  readonly message: ClaudeMessage
}

interface ToolRow {
  readonly kind: 'tool'
  readonly key: string
  call: ClaudeMessage | null
  result: ClaudeMessage | null
}

type Row = MessageRow | ToolRow

/** A tool call and the result echoed back for it read as one line, paired by tool-use id. */
const rows = computed<Row[]>(() => {
  const out: Row[] = []
  const byUseId = new Map<string, ToolRow>()
  let lastTool: ToolRow | null = null
  for (const message of props.messages) {
    if (message.role === 'TOOL_USE') {
      const row: ToolRow = { kind: 'tool', key: message.id, call: message, result: null }
      out.push(row)
      lastTool = row
      const useId = parseToolMeta(message.meta).toolUseId
      if (useId) byUseId.set(useId, row)
      continue
    }
    if (message.role === 'TOOL_RESULT') {
      const useId = parseToolMeta(message.meta).toolUseId
      const target = (useId && byUseId.get(useId)) || (lastTool && !lastTool.result ? lastTool : null)
      if (target) target.result = message
      else out.push({ kind: 'tool', key: message.id, call: null, result: message })
      continue
    }
    out.push({ kind: 'message', key: message.id, message })
  }
  return out
})

const scroller = ref<HTMLElement | null>(null)
let stick = true

function onScroll(): void {
  const element = scroller.value
  if (!element) return
  stick = element.scrollHeight - element.scrollTop - element.clientHeight < 48
}

function toBottom(): void {
  if (!stick) return
  void nextTick(() => {
    const element = scroller.value
    if (element) element.scrollTop = element.scrollHeight
  })
}

watch(() => props.messages.length, toBottom)
watch(() => props.working, toBottom)
</script>

<template>
  <div
    ref="scroller"
    class="min-h-0 flex-1 space-y-3 overflow-y-auto px-5 py-4"
    data-testid="claude-transcript"
    @scroll="onScroll"
  >
    <template v-for="row in rows" :key="row.key">
      <ClaudeToolCall v-if="row.kind === 'tool'" :call="row.call" :result="row.result" />
      <ClaudeMessageBubble v-else :message="row.message" />
    </template>

    <div
      v-if="working"
      class="flex items-center gap-2 text-[11.5px] text-text-subtle"
      data-testid="claude-working"
    >
      <span class="relative grid size-2.5 place-items-center" aria-hidden="true">
        <span class="absolute inline-flex size-2.5 animate-ping rounded-full bg-accent/50" />
        <span class="relative inline-flex size-1.5 rounded-full bg-accent" />
      </span>
      Claude is working
    </div>
  </div>
</template>
