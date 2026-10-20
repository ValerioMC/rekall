<script setup lang="ts">
import { computed } from 'vue'
import ElementGlyph from '@/features/diagram/components/canvas/ElementGlyph.vue'
import SourceExcerptView from '@/features/diagram/components/viewer/SourceExcerptView.vue'
import { baseOf, folderOf, spanLabel } from '@/features/diagram/lib/source-label'
import { EXTERNAL_STYLE } from '@/features/diagram/lib/element-style'
import type { WalkthroughBeat } from '@/features/diagram/lib/walkthrough/walkthrough-beats'
import type { SourceLocation } from '@/features/diagram/model/diagram'
import type { DiagramId } from '@/model/branded'

/**
 * What happens at the current beat: who calls whom with which method, why, what goes in and
 * what comes back, and the method's code open in a window. A call that has a sheet of its own
 * offers to step into it.
 */
const props = defineProps<{
  diagramId: DiagramId
  beat: WalkthroughBeat
  position: number
  count: number
  labels: ReadonlyMap<string, string>
  external: ReadonlySet<string>
  sources: readonly SourceLocation[]
  /** The title of the sheet this call opens, when it opens one. */
  opensTitle: string | null
}>()

const emit = defineEmits<{ stepInto: [] }>()

const MOTION_LABEL = { call: 'Call', reply: 'Returns', self: 'Inside' } as const

const ends = computed(() =>
  [props.beat.message.from, props.beat.message.to].map((id) => ({ id, label: props.labels.get(id) ?? id, external: props.external.has(id) }))
)
const receivesExternal = computed(() => props.beat.motion === 'call' && props.external.has(props.beat.message.to))
/** A reply shows only what it carries back; a call what it receives and what it gives back. */
const values = computed(() => {
  const { motion, input, output } = props.beat
  const shown = motion === 'reply' ? [{ name: 'Returned', text: output }] : [{ name: 'Input', text: input }, { name: 'Output', text: output }]
  return shown.filter((value): value is { name: string; text: string } => value.text !== null)
})
</script>

<template>
  <aside
    class="glass pointer-events-auto flex max-h-full w-[min(420px,calc(100vw-2rem))] flex-col overflow-hidden rounded-card border border-border shadow-lift"
    :class="`motion-${beat.motion}`"
    aria-labelledby="walkthrough-card-title"
    aria-live="polite"
    data-testid="walkthrough-card"
  >
    <header class="shrink-0 border-b border-border px-4 py-3">
      <p class="flex items-center gap-2 text-caption">
        <span class="beat-badge rounded-full px-2 py-0.5 font-semibold">{{ MOTION_LABEL[beat.motion] }}</span>
        <span class="font-mono tabular-nums text-text-subtle">Step {{ position + 1 }} of {{ count }}</span>
      </p>
      <h3 id="walkthrough-card-title" class="mt-2 break-words font-mono text-body font-semibold leading-snug text-text">{{ beat.message.text || '(no text)' }}</h3>
      <p class="mt-2 flex min-w-0 flex-wrap items-center gap-1.5 text-small">
        <template v-for="(end, index) in ends" :key="`${end.id}-${index}`">
          <svg v-if="index === 1" viewBox="0 0 16 16" class="size-3.5 shrink-0 text-text-subtle" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-label="to"><path d="M3 8h9.5M9 4.5 12.5 8 9 11.5" /></svg>
          <span class="inline-flex min-w-0 items-center gap-1 rounded-control border border-border bg-canvas px-1.5 py-0.5">
            <ElementGlyph v-if="end.external" :glyph="EXTERNAL_STYLE.glyph" :size="13" :style="{ color: EXTERNAL_STYLE.tint }" />
            <span class="truncate text-text">{{ end.label }}</span>
            <span v-if="end.external" class="text-micro text-text-subtle">external</span>
          </span>
        </template>
      </p>
    </header>

    <div class="flex min-h-0 flex-col gap-4 overflow-y-auto px-4 pb-5 pt-4">
      <p v-if="beat.explain" class="text-body leading-relaxed text-text" data-testid="walkthrough-explain">{{ beat.explain }}</p>
      <p v-if="receivesExternal" class="rounded-control border border-dashed border-border px-3 py-2 text-small leading-relaxed text-text-muted">
        This call leaves the project's code: what happens inside {{ labels.get(beat.message.to) ?? beat.message.to }} is not shown.
      </p>

      <dl v-if="values.length > 0" class="grid gap-2" data-testid="walkthrough-values">
        <div v-for="value in values" :key="value.name">
          <dt class="section-label mb-1">{{ value.name }}</dt>
          <dd class="max-h-[140px] overflow-auto whitespace-pre-wrap break-words rounded-control border border-border bg-canvas px-3 py-2 font-mono text-caption leading-relaxed text-text">{{ value.text }}</dd>
        </div>
      </dl>

      <button
        v-if="beat.opens !== null"
        class="step-into focus-ring flex w-full items-center gap-3 rounded-control border px-3 py-2.5 text-left transition-colors"
        data-testid="walkthrough-step-into"
        @click="emit('stepInto')"
      >
        <svg viewBox="0 0 16 16" class="size-4 shrink-0" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M4 3v6.5h8M9 6.5l3 3-3 3" /></svg>
        <span class="min-w-0 flex-1">
          <span class="block text-small font-semibold">Step into this call</span>
          <span v-if="opensTitle" class="block truncate text-caption opacity-80">{{ opensTitle }}</span>
        </span>
        <kbd>↵</kbd>
      </button>

      <section aria-labelledby="walkthrough-code">
        <h4 id="walkthrough-code" class="section-label mb-2">Code</h4>
        <p v-if="sources.length === 0" class="rounded-control border border-dashed border-border px-3 py-2.5 text-small leading-relaxed text-text-subtle">Not linked to a place in the code.</p>
        <div v-for="(source, index) in sources" :key="`${beat.key}:${source.file}:${source.startLine}:${index}`" class="code-window mb-2 overflow-hidden rounded-control border border-border" data-testid="walkthrough-code-window">
          <div class="flex items-center gap-2 border-b border-border bg-surface-raised px-2.5 py-1.5">
            <span class="flex gap-1" aria-hidden="true"><span v-for="dot in 3" :key="dot" class="size-2 rounded-full bg-border-strong" /></span>
            <span class="flex min-w-0 flex-1 font-mono text-caption" :title="source.file">
              <span class="truncate text-text-subtle">{{ folderOf(source.file) }}</span><span class="shrink-0 text-text">{{ baseOf(source.file) }}</span>
            </span>
            <span class="shrink-0 font-mono text-caption tabular-nums text-text-subtle">{{ spanLabel(source) }}</span>
          </div>
          <p v-if="source.symbol" class="border-b border-border bg-canvas px-2.5 py-1 font-mono text-micro text-text-subtle">in {{ source.symbol }}</p>
          <SourceExcerptView :diagram-id="diagramId" :source="source" class="rounded-none border-0" />
        </div>
      </section>
    </div>
  </aside>
</template>

<style scoped>
.beat-badge {
  color: var(--beat-tint);
  background: color-mix(in srgb, var(--beat-tint) 14%, transparent);
}

.motion-call,
.motion-self {
  --beat-tint: var(--color-accent);
}

.motion-reply {
  --beat-tint: var(--color-kind-data);
}

.step-into {
  color: var(--color-accent-strong);
  border-color: color-mix(in srgb, var(--color-accent) 45%, transparent);
  background: color-mix(in srgb, var(--color-accent) 8%, transparent);
}

.step-into:hover {
  background: color-mix(in srgb, var(--color-accent) 14%, transparent);
}
</style>
