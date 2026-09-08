<script setup lang="ts">
import { computed, ref } from 'vue'
import { parseTurnStats, type ClaudeMessage } from '@/model/claude'

const props = defineProps<{ message: ClaudeMessage }>()

interface Segment {
  readonly kind: 'text' | 'code'
  readonly text: string
  readonly lang?: string
}

const FENCE = /```([\w-]*)\n?([\s\S]*?)```/g

const segments = computed<Segment[]>(() => {
  const source = props.message.content ?? ''
  const out: Segment[] = []
  let cursor = 0
  for (const match of source.matchAll(FENCE)) {
    const at = match.index ?? 0
    if (at > cursor) out.push({ kind: 'text', text: source.slice(cursor, at) })
    out.push({ kind: 'code', text: (match[2] ?? '').replace(/\n$/, ''), lang: match[1] || undefined })
    cursor = at + match[0].length
  }
  if (cursor < source.length) out.push({ kind: 'text', text: source.slice(cursor) })
  return out.length ? out : [{ kind: 'text', text: source }]
})

const stats = computed(() =>
  props.message.role === 'RESULT' || props.message.role === 'ERROR'
    ? parseTurnStats(props.message.meta)
    : null
)

const statLine = computed(() => {
  const s = stats.value
  if (!s) return ''
  const parts: string[] = []
  if (typeof s.numTurns === 'number') parts.push(`${s.numTurns} turn${s.numTurns === 1 ? '' : 's'}`)
  if (typeof s.durationMs === 'number') parts.push(`${(s.durationMs / 1000).toFixed(1)}s`)
  if (typeof s.costUsd === 'number' && s.costUsd > 0) parts.push(`$${s.costUsd.toFixed(3)}`)
  return parts.join(' · ')
})

const toolOpen = ref(false)
</script>

<template>
  <!-- A prompt typed in the console -->
  <div v-if="message.role === 'USER'" class="flex justify-end" data-testid="claude-msg-user">
    <div
      class="max-w-[85%] rounded-[var(--radius-card)] rounded-br-sm border border-accent-line bg-accent-soft px-3.5 py-2 text-[13px] leading-relaxed text-text"
    >
      <p class="mb-1 text-[9.5px] font-semibold uppercase tracking-[0.08em] text-accent/70">you</p>
      <p class="whitespace-pre-wrap break-words">{{ message.content }}</p>
    </div>
  </div>

  <!-- A block of Claude's reply -->
  <div v-else-if="message.role === 'ASSISTANT'" class="flex" data-testid="claude-msg-assistant">
    <div class="min-w-0 max-w-[92%] text-[13px] leading-relaxed text-text-muted">
      <template v-for="(segment, index) in segments" :key="index">
        <pre
          v-if="segment.kind === 'code'"
          class="my-2 overflow-x-auto rounded-[var(--radius-control)] border border-border bg-surface p-3 font-mono text-[12px] leading-relaxed text-text"
        ><code>{{ segment.text }}</code></pre>
        <p v-else-if="segment.text.trim()" class="whitespace-pre-wrap break-words">{{ segment.text }}</p>
      </template>
    </div>
  </div>

  <!-- A tool call, its input tucked away until asked for -->
  <div
    v-else-if="message.role === 'TOOL_USE'"
    class="flex"
    data-testid="claude-msg-tool-use"
  >
    <div class="min-w-0 max-w-[92%]">
      <button
        class="focus-ring inline-flex items-center gap-1.5 rounded-full border border-border bg-surface px-2 py-0.5 font-mono text-[11px] text-text-subtle transition-colors hover:border-border-strong hover:text-text-muted"
        :aria-expanded="toolOpen"
        @click="toolOpen = !toolOpen"
      >
        <svg class="size-2.5" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <path
            d="M7.5 1.5a2.5 2.5 0 0 0-2.4 3.2L1.6 8.2a1.3 1.3 0 0 0 1.8 1.8l3.5-3.5A2.5 2.5 0 1 0 7.5 1.5Z"
            stroke="currentColor"
            stroke-width="1"
            stroke-linejoin="round"
          />
        </svg>
        {{ message.toolName ?? 'tool' }}
      </button>
      <pre
        v-if="toolOpen && message.content"
        class="mt-1.5 overflow-x-auto rounded-[var(--radius-control)] border border-border bg-surface p-2.5 font-mono text-[11px] leading-relaxed text-text-subtle"
      >{{ message.content }}</pre>
    </div>
  </div>

  <!-- What a tool returned -->
  <details
    v-else-if="message.role === 'TOOL_RESULT'"
    class="group/tr text-text-subtle"
    data-testid="claude-msg-tool-result"
  >
    <summary
      class="focus-ring inline-flex cursor-pointer list-none items-center gap-1 font-mono text-[11px] text-text-subtle transition-colors hover:text-text-muted"
    >
      <span class="transition-transform group-open/tr:rotate-90" aria-hidden="true">›</span>
      result
    </summary>
    <pre
      class="mt-1 overflow-x-auto rounded-[var(--radius-control)] border border-border bg-surface p-2.5 font-mono text-[11px] leading-relaxed"
    >{{ message.content }}</pre>
  </details>

  <!-- End-of-turn summary -->
  <div
    v-else-if="message.role === 'RESULT'"
    class="flex items-center gap-2 py-0.5 text-[10.5px] text-text-subtle"
    data-testid="claude-msg-result"
  >
    <span class="h-px flex-1 bg-border" aria-hidden="true" />
    <span class="font-mono">{{ statLine || 'turn complete' }}</span>
    <span class="h-px flex-1 bg-border" aria-hidden="true" />
  </div>

  <!-- A failure -->
  <div
    v-else-if="message.role === 'ERROR'"
    class="rounded-[var(--radius-control)] border border-danger/40 bg-danger-soft px-3 py-2 text-[12px] text-danger"
    data-testid="claude-msg-error"
  >
    {{ message.content || 'The turn ended with an error.' }}
  </div>

  <!-- A note from Rekall itself -->
  <p
    v-else
    class="text-center text-[10.5px] italic text-text-subtle"
    data-testid="claude-msg-system"
  >
    {{ message.content }}
  </p>
</template>
