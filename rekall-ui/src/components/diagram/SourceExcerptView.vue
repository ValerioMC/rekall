<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { fetchSourceExcerpt } from '@/api/diagrams.api'
import { highlightLine } from '@/common/diagram/code-highlight'
import { ApiError } from '@/api/client'
import type { DiagramId } from '@/model/branded'
import type { SourceExcerpt, SourceLocation } from '@/model/diagram'

/**
 * CONCEPT → CODE: the lines an element points at, read from the project folder when opened,
 * with a few lines either side for context and the span itself marked in the gutter.
 */
const props = defineProps<{ diagramId: DiagramId; source: SourceLocation }>()

const excerpt = ref<SourceExcerpt | null>(null)
const failure = ref<string | null>(null)

onMounted(async () => {
  try {
    excerpt.value = await fetchSourceExcerpt(props.diagramId, props.source)
  } catch (caught) {
    failure.value = caught instanceof ApiError ? caught.message : 'The code could not be read.'
  }
})

/** Highlighted line by line: a token never has to be closed across a line break. */
const lines = computed(() => {
  const found = excerpt.value
  if (!found) return []
  return found.lines.map((line) => ({
    number: line.number,
    tokens: highlightLine(line.text, found.language),
    marked:
      found.highlightStart !== null &&
      found.highlightEnd !== null &&
      line.number >= found.highlightStart &&
      line.number <= found.highlightEnd
  }))
})
</script>

<template>
  <div class="overflow-hidden rounded-[var(--radius-control)] border border-border bg-canvas" data-testid="source-excerpt">
    <div v-if="failure" class="px-3 py-2.5 text-[12px] leading-relaxed text-text-muted" role="status">{{ failure }}</div>
    <div v-else-if="!excerpt" class="skeleton-lines flex flex-col gap-1.5 px-3 py-3" role="status" aria-busy="true">
      <span class="sr-only">Loading the code</span>
      <span v-for="index in 5" :key="index" class="skeleton h-2.5 rounded" :style="{ width: `${40 + ((index * 23) % 50)}%` }" />
    </div>
    <div v-else class="max-h-[320px] overflow-auto py-1.5">
      <pre class="min-w-max font-mono text-[11px] leading-[1.65]"><code><span
        v-for="line in lines"
        :key="line.number"
        class="excerpt-line"
        :class="{ 'is-marked': line.marked }"
      ><span class="excerpt-number">{{ line.number }}</span><span class="excerpt-text"><span v-for="(token, index) in line.tokens" :key="index" :class="token.classes">{{ token.text }}</span></span>
</span></code></pre>
      <p v-if="excerpt.truncated" class="px-3 pt-1 text-[11px] text-text-subtle">Cut at 400 lines of {{ excerpt.totalLines }}.</p>
    </div>
  </div>
</template>

<style scoped>
.excerpt-line {
  position: relative;
  display: flex;
  padding-right: 12px;
}

.excerpt-number {
  width: 44px;
  flex-shrink: 0;
  padding-right: 12px;
  text-align: right;
  color: var(--color-text-subtle);
  font-variant-numeric: tabular-nums;
  user-select: none;
}

.excerpt-text {
  color: var(--color-text-muted);
  white-space: pre;
}

/* The span itself: an accent wash and a lit marker in the gutter, the same capsule a selected
   row carries, so "this is the part" reads the same everywhere. */
.is-marked {
  background: color-mix(in srgb, var(--color-accent) 7%, transparent);
}

.is-marked::before {
  content: '';
  position: absolute;
  left: 3px;
  top: 0;
  bottom: 0;
  width: 2px;
  background: var(--color-accent);
  box-shadow: 0 0 8px -1px var(--color-accent);
}

.is-marked .excerpt-number {
  color: var(--color-accent-strong);
}

.is-marked .excerpt-text {
  color: var(--color-text);
}
</style>
