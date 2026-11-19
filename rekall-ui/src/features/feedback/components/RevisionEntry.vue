<script setup lang="ts">
import { computed, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import { diffLines, foldUnchanged, type DiffRow } from '@/features/feedback/lib/revision-diff'
import { isWaiting, type Feedback } from '@/features/feedback/model/feedback'

/**
 * What a rewrite of the description changed, as a diff: removed lines muted red, added lines in
 * the accent, long kept runs folded. A pending revision can be discarded, which puts the earlier
 * text back.
 */
const props = defineProps<{ entry: Feedback }>()
const emit = defineEmits<{ discard: [entry: Feedback] }>()

const rows = computed<DiffRow[]>(() => foldUnchanged(diffLines(props.entry.quotedMarkdown ?? '', props.entry.bodyMarkdown)))
const opened = ref<Set<number>>(new Set())

function open(index: number): void {
  opened.value = new Set([...opened.value, index])
}

function sign(kind: DiffRow['kind']): string {
  if (kind === 'added') return '+'
  return kind === 'removed' ? '−' : ''
}
</script>

<template>
  <div data-testid="revision-entry">
    <div class="diff rounded-control border border-border bg-canvas font-mono text-small" role="group" aria-label="Changes to the description">
      <template v-for="(row, index) in rows" :key="index">
        <button
          v-if="row.kind === 'folded' && !opened.has(index)"
          type="button"
          class="focus-ring flex w-full items-center gap-2 px-2.5 py-1 text-left text-micro text-text-subtle hover:bg-surface"
          data-testid="revision-fold"
          @click="open(index)"
        >
          <span class="w-3 shrink-0 text-center" aria-hidden="true">⋯</span>
          {{ row.lines.length }} unchanged lines
        </button>
        <template v-else>
          <div
            v-for="(line, lineIndex) in row.lines"
            :key="`${index}-${lineIndex}`"
            class="diff-line flex gap-2 px-2.5 py-px"
            :class="row.kind === 'folded' ? 'diff-same' : `diff-${row.kind}`"
            :data-testid="`revision-${row.kind === 'folded' ? 'same' : row.kind}`"
          >
            <span class="w-3 shrink-0 select-none text-center" aria-hidden="true">{{ sign(row.kind) }}</span>
            <span class="min-w-0 flex-1 whitespace-pre-wrap break-words">{{ line }}</span>
            <span v-if="row.kind === 'added'" class="sr-only">added</span>
            <span v-else-if="row.kind === 'removed'" class="sr-only">removed</span>
          </div>
        </template>
      </template>
    </div>
    <div v-if="isWaiting(entry)" class="mt-2 flex items-center gap-2">
      <AppButton size="sm" variant="quiet" data-testid="revision-discard" @click="emit('discard', entry)">
        Discard and restore the earlier text
      </AppButton>
    </div>
  </div>
</template>

<style scoped>
.diff-same {
  color: var(--color-text-subtle);
}

.diff-removed {
  background: var(--color-danger-soft);
  color: color-mix(in srgb, var(--color-danger) 78%, var(--color-text-muted));
}

.diff-added {
  background: var(--color-accent-soft);
  color: var(--color-accent-strong);
}
</style>
