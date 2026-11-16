<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { highlightLine, type CodeToken } from '@/features/diagram/lib/code-highlight'
import { linkWords } from '@/features/diagram/lib/code-words'
import type { SourceExcerpt } from '@/features/diagram/model/diagram'

/**
 * A whole file, scrolled to the span a step marks. Only the lines in view are highlighted and
 * drawn; a called method that has a step of its own is a word to click.
 */
const props = defineProps<{
  excerpt: SourceExcerpt
  start: number | null
  end: number | null
  methods: ReadonlyMap<string, number>
  /** The step being shown, so the word that calls it reads as the target. */
  position: number
}>()

const emit = defineEmits<{ walk: [step: number] }>()

const LINE_HEIGHT = 22
const OVERSCAN = 40
const FALLBACK_HEIGHT = 640
const SPAN_LEAD = 0.22
const TALL_SPAN = 0.6
const TALL_SPAN_LEAD = 24

const scroller = ref<HTMLElement | null>(null)
const scrollTop = ref(0)
const viewportHeight = ref(FALLBACK_HEIGHT)
const highlighted = new Map<number, CodeToken[]>()

const firstRow = computed(() => Math.max(0, Math.floor(scrollTop.value / LINE_HEIGHT) - OVERSCAN))
const lastRow = computed(() => Math.min(props.excerpt.lines.length, Math.ceil((scrollTop.value + viewportHeight.value) / LINE_HEIGHT) + OVERSCAN))
const rows = computed(() =>
  props.excerpt.lines.slice(firstRow.value, lastRow.value).map((line) => ({
    number: line.number,
    marked: isMarked(line.number),
    segments: linkWords(tokensOf(line.number, line.text), props.methods)
  }))
)

function isMarked(number: number): boolean {
  return props.start !== null && number >= props.start && number <= (props.end ?? props.start)
}

function tokensOf(number: number, text: string): CodeToken[] {
  const known = highlighted.get(number)
  if (known) return known
  const tokens = highlightLine(text, props.excerpt.language)
  highlighted.set(number, tokens)
  return tokens
}

function onScroll(): void {
  const element = scroller.value
  if (!element) return
  scrollTop.value = element.scrollTop
  viewportHeight.value = element.clientHeight || FALLBACK_HEIGHT
}

function scrollToSpan(): void {
  const element = scroller.value
  if (!element || props.start === null) return
  const height = element.clientHeight || FALLBACK_HEIGHT
  const spanHeight = ((props.end ?? props.start) - props.start + 1) * LINE_HEIGHT
  const lead = spanHeight > height * TALL_SPAN ? TALL_SPAN_LEAD : height * SPAN_LEAD
  element.scrollTop = Math.max(0, (props.start - 1) * LINE_HEIGHT - lead)
  onScroll()
}

watch(
  () => props.excerpt,
  () => highlighted.clear()
)
watch(
  () => [props.excerpt, props.start, props.end],
  async () => {
    await nextTick()
    scrollToSpan()
  }
)
onMounted(scrollToSpan)
</script>

<template>
  <div ref="scroller" class="code-scroll min-h-0 flex-1 overflow-auto bg-canvas py-2" tabindex="0" aria-label="Source file" data-testid="code-file" @scroll.passive="onScroll">
    <div class="code-rows" :style="{ paddingTop: `${firstRow * LINE_HEIGHT}px`, paddingBottom: `${(excerpt.lines.length - lastRow) * LINE_HEIGHT}px` }">
      <div v-for="row in rows" :key="row.number" class="code-line" :class="{ 'is-marked': row.marked }" :data-line="row.number">
        <span class="code-number">{{ row.number }}</span>
        <span class="code-text"><template v-for="(segment, index) in row.segments" :key="index"><span v-if="segment.step === null" :class="segment.classes">{{ segment.text }}</span><span v-else role="button" class="code-link" :class="[segment.classes, { 'is-target': segment.step === position }]" :title="`Go to step ${segment.step + 1}`" data-testid="code-link" @click="emit('walk', segment.step)">{{ segment.text }}</span></template></span>
      </div>
    </div>
    <p v-if="excerpt.truncated" class="px-4 pt-2 text-caption text-text-subtle">Cut at {{ excerpt.lines.length }} lines of {{ excerpt.totalLines }}.</p>
  </div>
</template>

<style scoped>
.code-rows {
  min-width: max-content;
  font-family: var(--font-mono);
  font-size: var(--text-body);
}

.code-line {
  position: relative;
  display: flex;
  height: 22px;
  align-items: center;
  padding-right: 24px;
}

.code-number {
  width: 56px;
  flex-shrink: 0;
  padding-right: 16px;
  text-align: right;
  color: var(--color-text-subtle);
  font-variant-numeric: tabular-nums;
  user-select: none;
}

.code-text {
  white-space: pre;
  color: var(--color-text-muted);
}

/* The span itself: an accent wash and a lit marker in the gutter, the same capsule a selected row carries. */
.is-marked {
  background: color-mix(in srgb, var(--color-accent) 7%, transparent);
  transition: background var(--motion-move) ease-out;
}

.is-marked::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 2px;
  background: var(--color-accent);
  box-shadow: 0 0 8px -1px var(--color-accent);
}

.is-marked .code-number {
  color: var(--color-accent-strong);
}

.is-marked .code-text {
  color: var(--color-text);
}

.code-link {
  cursor: pointer;
  border-radius: 2px;
  text-decoration: underline dotted var(--color-border-field);
  text-decoration-thickness: 1px;
  text-underline-offset: 4px;
  transition: background var(--motion-quick) ease;
}

.code-link.is-target {
  text-decoration-color: var(--color-accent);
}

@media (hover: hover) and (pointer: fine) {
  .code-link:hover {
    background: var(--color-surface-hover);
    text-decoration-style: solid;
    text-decoration-color: var(--color-text);
  }
}
</style>
