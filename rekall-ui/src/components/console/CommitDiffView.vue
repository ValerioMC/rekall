<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{ diff: string | null }>()

type DiffLine = { text: string; kind: 'add' | 'remove' | 'hunk' | 'meta' | 'context' }

function kindOf(line: string): DiffLine['kind'] {
  if (line.startsWith('+++') || line.startsWith('---')) return 'meta'
  if (line.startsWith('+')) return 'add'
  if (line.startsWith('-')) return 'remove'
  if (line.startsWith('@@')) return 'hunk'
  if (line.startsWith('diff --git') || line.startsWith('index ')) return 'meta'
  return 'context'
}

const lines = computed<DiffLine[]>(() =>
  (props.diff ?? '').split('\n').map((text) => ({ text, kind: kindOf(text) }))
)
</script>

<template>
  <p v-if="!diff" class="px-1 py-1 text-[11px] text-text-subtle" data-testid="commit-diff-empty">
    No diff recorded for this commit.
  </p>
  <pre
    v-else
    class="max-h-[320px] min-w-0 overflow-auto rounded-[var(--radius-control)] border border-border bg-canvas px-2.5 py-2 font-mono text-[11px] leading-relaxed"
    data-testid="commit-diff-view"
  ><code
    v-for="(line, index) in lines"
    :key="index"
    class="block whitespace-pre"
    :class="{
      'bg-safe/10 text-safe': line.kind === 'add',
      'bg-danger/10 text-danger': line.kind === 'remove',
      'text-accent': line.kind === 'hunk',
      'text-text-subtle': line.kind === 'meta',
      'text-text-muted': line.kind === 'context'
    }"
  >{{ line.text }}</code></pre>
</template>
