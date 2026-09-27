<script setup lang="ts">
import { computed } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import { baseOf, coverageOf, filesOf, folderOf } from '@/common/diagram/code-index'
import { relativeTime } from '@/common/format/relative-time'
import type { Diagram, Provenance } from '@/model/diagram'

/**
 * The diagram as a whole, when no element is selected: the question it answers, how much of
 * it can be followed to code, and CODE → CONCEPT per file: pick a file to light its elements.
 */
const props = defineProps<{ diagram: Diagram; focusedFile: string | null }>()

const emit = defineEmits<{ focusFile: [file: string | null]; hoverFile: [file: string | null]; remove: [] }>()

const coverage = computed(() => coverageOf(props.diagram.graph))
const percent = computed(() => (coverage.value.total === 0 ? 0 : Math.round((coverage.value.traced / coverage.value.total) * 100)))
const files = computed(() => filesOf(props.diagram.graph))

const ORDER: readonly Provenance[] = ['observed', 'inferred', 'documented', 'stated']
const provenance = computed(() => {
  const counts = new Map<Provenance | 'unsaid', number>()
  for (const node of props.diagram.graph.nodes) {
    const key = node.provenance ?? 'unsaid'
    counts.set(key, (counts.get(key) ?? 0) + 1)
  }
  return [...ORDER, 'unsaid' as const].filter((key) => counts.has(key)).map((key) => ({ key, count: counts.get(key)! }))
})
</script>

<template>
  <div class="flex flex-col gap-6" data-testid="diagram-overview">
    <section>
      <p class="eyebrow mb-1.5">The question</p>
      <p class="whitespace-pre-line text-[13.5px] leading-relaxed text-text">{{ diagram.question || 'No question was recorded.' }}</p>
      <p class="mt-2 text-[11.5px] text-text-subtle">
        Written {{ relativeTime(diagram.createdAt) }}<template v-if="diagram.updatedAt !== diagram.createdAt">, revised {{ relativeTime(diagram.updatedAt) }}</template>
      </p>
    </section>

    <section aria-labelledby="overview-trace">
      <h4 id="overview-trace" class="section-label mb-2 flex items-baseline justify-between">
        Traced to code
        <span class="font-mono text-[11.5px] font-normal tabular-nums text-text-muted">{{ coverage.traced }} / {{ coverage.total }}</span>
      </h4>
      <div class="h-1.5 overflow-hidden rounded-full bg-surface-raised" role="meter" :aria-valuenow="percent" aria-valuemin="0" aria-valuemax="100" aria-label="Elements traced to code">
        <div class="h-full rounded-full bg-accent/80 transition-[width] duration-500" :style="{ width: `${percent}%` }" />
      </div>
      <div class="mt-2.5 flex flex-wrap gap-1.5">
        <span v-for="entry in provenance" :key="entry.key" class="rounded-full border border-border px-2 py-0.5 text-[11px] text-text-muted">
          {{ entry.key === 'unsaid' ? 'no provenance' : entry.key }} <span class="font-mono tabular-nums text-text-subtle">{{ entry.count }}</span>
        </span>
      </div>
    </section>

    <section aria-labelledby="overview-files">
      <h4 id="overview-files" class="section-label mb-1">Files</h4>
      <p class="mb-2 text-[11.5px] text-text-subtle">Pick a file to light the elements it implements.</p>
      <p v-if="files.length === 0" class="rounded-[var(--radius-control)] border border-dashed border-border px-3 py-2.5 text-[12px] text-text-subtle">
        No element points at code yet.
      </p>
      <ul v-else class="flex flex-col">
        <li v-for="entry in files" :key="entry.file">
          <button
            class="focus-ring flex h-(--spacing-row) w-full items-center gap-2 rounded-[var(--radius-control)] px-2.5 text-left transition-colors"
            :class="focusedFile === entry.file ? 'selected-row text-text' : 'text-text-muted hover:bg-surface-raised hover:text-text'"
            :aria-pressed="focusedFile === entry.file"
            data-testid="overview-file"
            @pointerenter="emit('hoverFile', entry.file)"
            @pointerleave="emit('hoverFile', null)"
            @click="emit('focusFile', focusedFile === entry.file ? null : entry.file)"
          >
            <span class="flex min-w-0 flex-1 font-mono text-[11.5px]" :title="entry.file">
              <span class="truncate text-text-subtle">{{ folderOf(entry.file) }}</span><span class="shrink-0">{{ baseOf(entry.file) }}</span>
            </span>
            <span class="shrink-0 font-mono text-[11px] tabular-nums text-text-subtle">{{ entry.nodeIds.length }}</span>
          </button>
        </li>
      </ul>
    </section>

    <div class="border-t border-border pt-4">
      <AppButton variant="danger-quiet" size="sm" data-testid="diagram-delete" @click="emit('remove')">Delete diagram</AppButton>
    </div>
  </div>
</template>
