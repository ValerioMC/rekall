<script setup lang="ts">
import { ref, watch } from 'vue'
import CallTreeView from './CallTreeView.vue'
import SourceExcerptView from './SourceExcerptView.vue'
import ElementGlyph from '@/features/diagram/components/canvas/ElementGlyph.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import { baseOf, folderOf, spanLabel } from '@/features/diagram/lib/source-label'
import type { CallNode } from '@/features/diagram/lib/call-tree'
import type { Connection, ElementDetail } from '@/features/diagram/lib/element-detail'
import type { CodeLinks, SourceLocation } from '@/features/diagram/model/diagram'
import type { DiagramId } from '@/model/branded'

/**
 * The element the reader picked, floating over the canvas: the code it is implemented by, opened
 * in place, and in a sequence the calls that go through it; in any other sheet, its connections.
 */
const props = defineProps<{
  diagramId: DiagramId
  detail: ElementDetail
  sources: readonly SourceLocation[]
  calls: readonly CallNode[] | null
  connections: { readonly outgoing: readonly Connection[]; readonly incoming: readonly Connection[] } | null
  labels: ReadonlyMap<string, string>
  links: CodeLinks
}>()

const emit = defineEmits<{ close: []; select: [key: string] }>()

const open = ref<ReadonlySet<number>>(new Set([0]))
watch(
  () => props.detail.key,
  () => (open.value = new Set([0]))
)

function toggleSource(index: number): void {
  const next = new Set(open.value)
  if (next.has(index)) next.delete(index)
  else next.add(index)
  open.value = next
}
</script>

<template>
  <aside
    class="glass pointer-events-auto flex max-h-full w-[min(400px,calc(100vw-2rem))] flex-col overflow-hidden rounded-card border border-border shadow-lift"
    :style="{ '--tint': detail.look.tint }"
    aria-labelledby="element-panel-title"
    data-testid="element-panel"
  >
    <header class="flex shrink-0 items-start gap-3 border-b border-border px-4 py-3">
      <span class="mt-0.5 grid size-8 shrink-0 place-items-center rounded-full border" :style="{ color: detail.look.tint, borderColor: `color-mix(in srgb, ${detail.look.tint} 34%, transparent)`, background: `color-mix(in srgb, ${detail.look.tint} 12%, var(--color-canvas))` }">
        <ElementGlyph :glyph="detail.look.glyph" />
      </span>
      <span class="min-w-0 flex-1">
        <span class="block text-caption font-medium" :style="{ color: detail.look.tint }">{{ detail.look.label }}</span>
        <h3 id="element-panel-title" class="mt-0.5 break-words text-title font-semibold leading-snug tracking-[-0.01em] text-text">{{ detail.title }}</h3>
        <span v-if="detail.subtitle" class="mt-0.5 block truncate font-mono text-caption text-text-subtle">{{ detail.subtitle }}</span>
      </span>
      <button class="focus-ring grid size-7 shrink-0 place-items-center rounded-control text-text-subtle transition-colors hover:bg-surface-raised hover:text-text" aria-label="Close (Esc)" title="Close (Esc)" data-testid="element-panel-close" @click="emit('close')">
        <CloseGlyph />
      </button>
    </header>

    <div class="flex min-h-0 flex-col gap-5 overflow-y-auto px-4 pb-5 pt-4">
      <section aria-labelledby="element-code">
        <h4 id="element-code" class="section-label mb-2 flex items-baseline gap-2">
          Code <span class="font-mono text-caption font-normal text-text-subtle">{{ sources.length }}</span>
        </h4>
        <p v-if="sources.length === 0" class="rounded-control border border-dashed border-border px-3 py-2.5 text-small leading-relaxed text-text-subtle">
          Not linked to a place in the code.
        </p>
        <ul v-else class="flex flex-col gap-2">
          <li v-for="(source, index) in sources" :key="`${source.file}:${source.startLine}:${index}`">
            <button
              class="focus-ring flex w-full items-center gap-2 rounded-control px-2 py-1.5 text-left transition-colors hover:bg-surface-raised"
              :aria-expanded="open.has(index)"
              data-testid="source-toggle"
              @click="toggleSource(index)"
            >
              <svg viewBox="0 0 12 12" class="size-3 shrink-0 text-text-subtle transition-transform duration-(--motion-quick)" :class="open.has(index) ? 'rotate-90' : ''" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m4.5 2.5 3.5 3.5-3.5 3.5" /></svg>
              <span class="flex min-w-0 flex-1 font-mono text-caption" :title="source.file">
                <span class="truncate text-text-subtle">{{ folderOf(source.file) }}</span><span class="shrink-0 text-text">{{ baseOf(source.file) }}</span>
              </span>
              <span class="shrink-0 font-mono text-caption tabular-nums text-text-subtle">{{ spanLabel(source) }}</span>
            </button>
            <p v-if="source.symbol" class="ml-7 font-mono text-caption text-text-subtle">in {{ source.symbol }}</p>
            <div v-if="open.has(index)" class="mt-1.5">
              <SourceExcerptView :diagram-id="diagramId" :source="source" />
            </div>
          </li>
        </ul>
      </section>

      <section v-if="calls" aria-labelledby="element-calls">
        <h4 id="element-calls" class="section-label mb-2">Call tree</h4>
        <p v-if="calls.length === 0" class="px-2 text-small text-text-subtle">No call goes through here.</p>
        <div v-else role="tree" aria-labelledby="element-calls">
          <CallTreeView :nodes="calls" :labels="labels" :links="links" :selected="detail.key" @select="emit('select', $event)" />
        </div>
      </section>

      <section v-if="connections && connections.outgoing.length + connections.incoming.length > 0" aria-labelledby="element-connections">
        <h4 id="element-connections" class="section-label mb-2">Connections</h4>
        <template v-for="group in [{ name: 'Goes to', rows: connections.outgoing }, { name: 'Comes from', rows: connections.incoming }]" :key="group.name">
          <div v-if="group.rows.length > 0" class="mb-2">
            <p class="mb-0.5 px-2 text-caption text-text-subtle">{{ group.name }}</p>
            <button
              v-for="row in group.rows"
              :key="row.edgeKey"
              class="focus-ring flex w-full items-center gap-2 rounded-control px-2 py-1.5 text-left transition-colors hover:bg-surface-raised"
              data-testid="connection-row"
              @click="emit('select', row.otherKey)"
            >
              <ElementGlyph :glyph="row.look.glyph" :style="{ color: row.look.tint }" class="shrink-0" />
              <span class="min-w-0 flex-1 truncate text-small text-text">{{ row.otherTitle }}</span>
              <span v-if="row.label" class="max-w-[120px] shrink-0 truncate rounded-full border border-border px-1.5 text-micro text-text-muted" :title="row.label">{{ row.label }}</span>
            </button>
          </div>
        </template>
      </section>
    </div>
  </aside>
</template>
