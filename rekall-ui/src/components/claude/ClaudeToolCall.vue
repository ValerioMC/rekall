<script setup lang="ts">
import { computed } from 'vue'
import { claudeToolDetail, parseToolMeta, type ClaudeMessage } from '@/model/claude'

const props = defineProps<{ call: ClaudeMessage | null; result: ClaudeMessage | null }>()

const toolName = computed(() => props.call?.toolName ?? 'tool')

const detail = computed(() =>
  claudeToolDetail(props.call?.toolName ?? null, props.call?.content ?? null)
)

const failed = computed(() => parseToolMeta(props.result?.meta ?? null).error === true)

/** A short read of what the call returned, shown on the row so it need not be opened. */
const outcome = computed<string | null>(() => {
  if (!props.result) return null
  if (failed.value) return 'error'
  const body = (props.result.content ?? '').trim()
  if (!body) return 'ok'
  const lines = body.split('\n')
  const first = (lines[0] ?? '').trim()
  const more = lines.length > 1 ? ` +${lines.length - 1}` : ''
  return `${first.length > 44 ? `${first.slice(0, 43)}…` : first}${more}`
})
</script>

<template>
  <details class="group/tc" data-testid="claude-tool-call">
    <summary
      class="focus-ring flex cursor-pointer list-none items-center gap-2 py-0.5 font-mono text-[11px] text-text-subtle"
    >
      <span class="shrink-0 transition-transform group-open/tc:rotate-90" aria-hidden="true">›</span>
      <span
        class="inline-flex shrink-0 items-center gap-1.5 rounded-full border border-border bg-surface px-2 py-0.5 text-text-muted"
      >
        <svg class="size-2.5" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <path
            d="M7.5 1.5a2.5 2.5 0 0 0-2.4 3.2L1.6 8.2a1.3 1.3 0 0 0 1.8 1.8l3.5-3.5A2.5 2.5 0 1 0 7.5 1.5Z"
            stroke="currentColor"
            stroke-width="1"
            stroke-linejoin="round"
          />
        </svg>
        {{ toolName }}
      </span>
      <span v-if="detail" class="min-w-0 flex-1 truncate text-text-muted" :title="detail">
        {{ detail }}
      </span>
      <span v-else class="flex-1" />
      <span
        v-if="outcome"
        class="shrink-0 truncate text-right"
        :class="failed ? 'text-danger' : 'text-text-subtle/80'"
        style="max-width: 45%"
      >
        {{ outcome }}
      </span>
    </summary>

    <div class="mt-1.5 space-y-1.5 pl-4">
      <pre
        v-if="call?.content"
        class="overflow-x-auto rounded-[var(--radius-control)] border border-border bg-surface p-2.5 font-mono text-[11px] leading-relaxed text-text-subtle"
      >{{ call.content }}</pre>
      <pre
        v-if="result?.content"
        class="overflow-x-auto rounded-[var(--radius-control)] border p-2.5 font-mono text-[11px] leading-relaxed"
        :class="
          failed
            ? 'border-danger/40 bg-danger-soft text-danger'
            : 'border-border bg-surface text-text-subtle'
        "
      >{{ result.content }}</pre>
    </div>
  </details>
</template>
