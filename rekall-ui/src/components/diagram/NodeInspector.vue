<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import NodeKindGlyph from '@/components/diagram/NodeKindGlyph.vue'
import SourceExcerptView from '@/components/diagram/SourceExcerptView.vue'
import TraceMark from '@/components/diagram/TraceMark.vue'
import CopyGlyph from '@/components/ui/CopyGlyph.vue'
import { humanise, kindStyle, relationVerb } from '@/common/diagram/kind-style'
import { baseOf, folderOf, spanLabel } from '@/common/diagram/code-index'
import { containerMap } from '@/common/diagram/graph-projection'
import type { Diagram, GraphNode, Provenance } from '@/model/diagram'

/**
 * One element in full: what it is, how far to trust it, the code that implements it (opened
 * in place), what holds it and what it holds, and every relation, each a way to walk on.
 */
const props = defineProps<{ diagram: Diagram; node: GraphNode; folded: boolean; foldable: boolean }>()

const emit = defineEmits<{ select: [id: string]; toggleFold: [id: string] }>()

const style = computed(() => kindStyle(props.node.kind))
const byId = computed(() => new Map(props.diagram.graph.nodes.map((node) => [node.id, node])))
const parents = computed(() => containerMap(props.diagram.graph))
const container = computed(() => {
  const id = parents.value.get(props.node.id)
  return id ? byId.value.get(id) ?? null : null
})
const children = computed(() =>
  props.diagram.graph.edges
    .filter((edge) => edge.relation === 'contains' && edge.from === props.node.id)
    .map((edge) => byId.value.get(edge.to))
    .filter((node): node is GraphNode => node !== undefined)
)

interface RelationRow {
  readonly key: string
  readonly relation: string
  readonly label: string | null
  readonly other: GraphNode
}

const outgoing = computed<RelationRow[]>(() => relations('out'))
const incoming = computed<RelationRow[]>(() => relations('in'))

function relations(direction: 'in' | 'out'): RelationRow[] {
  return props.diagram.graph.edges
    .filter((edge) => edge.relation !== 'contains' && (direction === 'out' ? edge.from : edge.to) === props.node.id)
    .map((edge) => ({
      key: edge.id,
      relation: edge.relation,
      label: edge.label,
      other: byId.value.get(direction === 'out' ? edge.to : edge.from)!
    }))
    .filter((row) => row.other !== undefined)
}

const PROVENANCE: Readonly<Record<Provenance, string>> = {
  observed: 'Observed in the code',
  inferred: 'Inferred from the code',
  documented: 'From comments or docs',
  stated: 'Stated by a person'
}

const trust = computed(() => {
  const parts: string[] = []
  if (props.node.provenance) parts.push(PROVENANCE[props.node.provenance])
  if (props.node.confidence !== null) parts.push(`${Math.round(props.node.confidence * 100)}% confident`)
  return parts.length > 0 ? parts.join(' · ') : 'No provenance given'
})

const metadata = computed(() =>
  Object.entries(props.node.metadata).map(([key, value]) => ({
    key,
    value: typeof value === 'string' ? value : JSON.stringify(value)
  }))
)

const open = ref<ReadonlySet<number>>(new Set([0]))
watch(
  () => props.node.id,
  () => (open.value = new Set([0]))
)

function toggleSource(index: number): void {
  const next = new Set(open.value)
  if (next.has(index)) next.delete(index)
  else next.add(index)
  open.value = next
}

const copied = ref(false)
async function copyId(): Promise<void> {
  await navigator.clipboard?.writeText(props.node.id)
  copied.value = true
  window.setTimeout(() => (copied.value = false), 1200)
}
</script>

<template>
  <article class="flex flex-col gap-5" data-testid="node-inspector" :style="{ '--tint': style.tint }">
    <header>
      <div class="flex items-center gap-2.5">
        <span class="grid size-8 shrink-0 place-items-center rounded-full border" :style="{ color: style.tint, borderColor: `color-mix(in srgb, ${style.tint} 34%, transparent)`, background: `color-mix(in srgb, ${style.tint} 12%, var(--color-canvas))` }">
          <NodeKindGlyph :glyph="style.glyph" />
        </span>
        <span class="text-[12px] font-medium" :style="{ color: style.tint }">{{ style.label }}</span>
        <button class="focus-ring ml-auto flex items-center gap-1.5 rounded px-1 font-mono text-[11px] text-text-subtle hover:text-text" :title="`Copy the id ${node.id}`" @click="copyId">
          <span class="max-w-[140px] truncate">{{ node.id }}</span>
          <CopyGlyph :copied="copied" />
        </button>
      </div>
      <h3 class="mt-3 text-[18px] font-semibold leading-snug tracking-[-0.012em] text-text">{{ node.title }}</h3>
      <div class="mt-2.5 flex items-center gap-2.5">
        <TraceMark :confidence="node.confidence" :provenance="node.provenance" :traced="node.sources.length > 0" :tint="style.tint" :size="26" />
        <span class="text-[12px] text-text-muted">{{ trust }}</span>
      </div>
      <p v-if="node.description" class="mt-3 whitespace-pre-line text-[13px] leading-relaxed text-text-muted">{{ node.description }}</p>
    </header>

    <section aria-labelledby="inspector-code">
      <h4 id="inspector-code" class="section-label mb-2 flex items-baseline gap-2">
        Implemented by <span class="font-mono text-[11px] font-normal text-text-subtle">{{ node.sources.length }}</span>
      </h4>
      <p v-if="node.sources.length === 0" class="rounded-[var(--radius-control)] border border-dashed border-border px-3 py-2.5 text-[12px] leading-relaxed text-text-subtle">
        Not tied to a place in the code: this is how the diagram reads the code, not a part of it.
      </p>
      <ul v-else class="flex flex-col gap-2">
        <li v-for="(source, index) in node.sources" :key="`${source.file}:${source.startLine}:${index}`">
          <button
            class="focus-ring group flex w-full items-center gap-2 rounded-[var(--radius-control)] px-2 py-1.5 text-left transition-colors hover:bg-surface-raised"
            :aria-expanded="open.has(index)"
            data-testid="source-toggle"
            @click="toggleSource(index)"
          >
            <svg viewBox="0 0 12 12" class="size-3 shrink-0 text-text-subtle transition-transform duration-150" :class="open.has(index) ? 'rotate-90' : ''" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m4.5 2.5 3.5 3.5-3.5 3.5" /></svg>
            <span class="flex min-w-0 flex-1 font-mono text-[11.5px]" :title="source.file">
              <span class="truncate text-text-subtle">{{ folderOf(source.file) }}</span><span class="shrink-0 text-text">{{ baseOf(source.file) }}</span>
            </span>
            <span class="shrink-0 font-mono text-[11px] tabular-nums text-text-subtle">{{ spanLabel(source) }}</span>
          </button>
          <p v-if="source.symbol" class="ml-7 font-mono text-[11px] text-text-subtle">in {{ source.symbol }}</p>
          <div v-if="open.has(index)" class="mt-1.5">
            <SourceExcerptView :diagram-id="diagram.id" :source="source" />
          </div>
        </li>
      </ul>
    </section>

    <section v-if="container || children.length > 0" aria-labelledby="inspector-structure">
      <h4 id="inspector-structure" class="section-label mb-2">Structure</h4>
      <button v-if="container" class="focus-ring mb-2 flex w-full items-center gap-2 rounded-[var(--radius-control)] px-2 py-1.5 text-left text-[12.5px] text-text-muted transition-colors hover:bg-surface-raised hover:text-text" @click="emit('select', container.id)">
        <span class="w-14 shrink-0 text-[11px] text-text-subtle">inside</span>
        <NodeKindGlyph :glyph="kindStyle(container.kind).glyph" :style="{ color: kindStyle(container.kind).tint }" class="shrink-0" />
        <span class="min-w-0 truncate">{{ container.title }}</span>
      </button>
      <div v-if="children.length > 0">
        <div class="mb-1 flex items-center gap-2 px-2">
          <span class="text-[11px] text-text-subtle">holds {{ children.length }}</span>
          <button v-if="foldable" class="focus-ring ml-auto rounded px-1.5 py-0.5 text-[11px] text-text-subtle hover:bg-surface-raised hover:text-text" data-testid="inspector-fold" @click="emit('toggleFold', node.id)">
            {{ folded ? 'Unfold' : 'Fold' }}
          </button>
        </div>
        <button
          v-for="child in children"
          :key="child.id"
          class="focus-ring flex w-full items-center gap-2 rounded-[var(--radius-control)] px-2 py-1.5 text-left text-[12.5px] text-text-muted transition-colors hover:bg-surface-raised hover:text-text"
          @click="emit('select', child.id)"
        >
          <NodeKindGlyph :glyph="kindStyle(child.kind).glyph" :style="{ color: kindStyle(child.kind).tint }" class="shrink-0" />
          <span class="min-w-0 truncate">{{ child.title }}</span>
        </button>
      </div>
    </section>

    <section v-if="outgoing.length + incoming.length > 0" aria-labelledby="inspector-relations">
      <h4 id="inspector-relations" class="section-label mb-2">Relations</h4>
      <template v-for="group in [{ name: 'Goes to', rows: outgoing, arrow: '→' }, { name: 'Comes from', rows: incoming, arrow: '←' }]" :key="group.name">
        <div v-if="group.rows.length > 0" class="mb-2">
          <p class="mb-0.5 px-2 text-[11px] text-text-subtle">{{ group.name }}</p>
          <button
            v-for="row in group.rows"
            :key="row.key"
            class="focus-ring flex w-full items-center gap-2 rounded-[var(--radius-control)] px-2 py-1.5 text-left transition-colors hover:bg-surface-raised"
            data-testid="relation-row"
            @click="emit('select', row.other.id)"
          >
            <span class="w-[68px] shrink-0 truncate text-[11px] text-text-subtle" :title="humanise(row.relation)">{{ relationVerb(row.relation) }}</span>
            <NodeKindGlyph :glyph="kindStyle(row.other.kind).glyph" :style="{ color: kindStyle(row.other.kind).tint }" class="shrink-0" />
            <span class="min-w-0 flex-1 truncate text-[12.5px] text-text">{{ row.other.title }}</span>
            <span v-if="row.label" class="max-w-[90px] shrink-0 truncate rounded-full border border-border px-1.5 text-[10.5px] text-text-muted" :title="row.label">{{ row.label }}</span>
          </button>
        </div>
      </template>
    </section>

    <section v-if="metadata.length > 0" aria-labelledby="inspector-metadata">
      <h4 id="inspector-metadata" class="section-label mb-2">Details</h4>
      <dl class="grid grid-cols-[minmax(0,110px)_minmax(0,1fr)] gap-x-3 gap-y-1.5 px-2 text-[12px]">
        <template v-for="entry in metadata" :key="entry.key">
          <dt class="truncate text-text-subtle" :title="entry.key">{{ entry.key }}</dt>
          <dd class="break-words font-mono text-[11.5px] text-text-muted">{{ entry.value }}</dd>
        </template>
      </dl>
    </section>
  </article>
</template>
