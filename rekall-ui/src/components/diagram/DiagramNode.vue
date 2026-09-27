<script setup lang="ts">
import { computed } from 'vue'
import NodeKindGlyph from '@/components/diagram/NodeKindGlyph.vue'
import TraceMark from '@/components/diagram/TraceMark.vue'
import { kindStyle } from '@/common/diagram/kind-style'
import { detailPath, shapePath } from '@/common/diagram/node-shape'
import type { Emphasis } from '@/common/diagram/emphasis'
import type { PlacedNode } from '@/common/diagram/graph-layout'
import type { ProjectedNode } from '@/common/diagram/graph-projection'

/**
 * One element on the canvas. The outline is the kind, the socket on the left repeats it as a
 * glyph, the trace mark on the right says how far to trust it. A folded node shows the pieces
 * it holds as two cards stacked behind it.
 */
const props = defineProps<{
  placed: PlacedNode
  item: ProjectedNode
  emphasis: Emphasis
  arriveDelay: number
}>()

const emit = defineEmits<{ select: [id: string]; toggleFold: [id: string] }>()

const style = computed(() => kindStyle(props.item.node.kind))
const outline = computed(() => shapePath(style.value.shape, props.placed.width, props.placed.height))
const detail = computed(() => detailPath(style.value.shape, props.placed.width, props.placed.height))
const isCode = computed(() => style.value.shape === 'block')

/** Kind label and title lines, centred as one block in the box. */
const textTop = computed(() => (props.placed.height - (15 + props.placed.lines.length * 16)) / 2 + 1)
const socketY = computed(() => props.placed.height / 2)

const label = computed(() => `${style.value.label}: ${props.item.node.title}`)

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Enter' || event.key === ' ') {
    event.preventDefault()
    emit('select', props.item.id)
  }
}
</script>

<template>
  <g
    class="diagram-node"
    :class="[`is-${emphasis}`, { 'is-code': isCode }]"
    :style="{
      '--tint': style.tint,
      transform: `translate(${placed.x}px, ${placed.y}px)`,
      '--arrive-delay': `${arriveDelay}ms`
    }"
    role="button"
    tabindex="0"
    :aria-label="label"
    :aria-pressed="emphasis === 'selected'"
    :data-node-id="item.id"
    @click.stop="emit('select', item.id)"
    @keydown="onKeydown"
  >
    <g class="node-arrive">
      <template v-if="item.collapsed">
        <path :d="outline" class="node-stack" transform="translate(8 8)" />
        <path :d="outline" class="node-stack" transform="translate(4 4)" />
      </template>
      <path :d="outline" class="node-halo" />
      <path :d="outline" class="node-shade" transform="translate(0 3)" />
      <path :d="outline" class="node-body" />
      <path v-if="detail" :d="detail" class="node-detail" />

      <circle cx="24" :cy="socketY" r="13" class="node-socket" />
      <NodeKindGlyph :glyph="style.glyph" :x="16" :y="socketY - 8" class="node-glyph" />

      <text :x="46" :y="textTop + 9" class="node-kind">{{ style.label.toLowerCase() }}</text>
      <text
        v-for="(line, index) in placed.lines"
        :key="index"
        :x="46"
        :y="textTop + 27 + index * 16"
        class="node-title"
      >
        {{ line }}
      </text>

      <TraceMark
        :x="placed.width - 27"
        :y="7"
        :confidence="item.node.confidence"
        :provenance="item.node.provenance"
        :traced="item.node.sources.length > 0"
        :tint="style.tint"
      />

      <g
        v-if="item.collapsed"
        class="node-fold"
        :transform="`translate(${placed.width - 52} ${placed.height - 8})`"
        role="button"
        :aria-label="`Unfold ${item.hiddenCount} elements`"
        @click.stop="emit('toggleFold', item.id)"
      >
        <rect width="36" height="16" rx="8" />
        <text x="18" y="11.5" text-anchor="middle">+{{ item.hiddenCount }}</text>
      </g>
    </g>
  </g>
</template>

<style scoped>
.diagram-node {
  cursor: pointer;
  outline: none;
  transition:
    transform 280ms cubic-bezier(0.16, 1, 0.3, 1),
    opacity 160ms ease;
}

.node-arrive {
  animation: node-arrive 320ms cubic-bezier(0.16, 1, 0.3, 1) both;
  animation-delay: var(--arrive-delay);
  transform-box: fill-box;
  transform-origin: center;
}

@keyframes node-arrive {
  from {
    opacity: 0;
    transform: translateY(6px) scale(0.97);
  }
}

/* Recessed like a slot in the canvas rather than floating: a hard shade 3px down, no blur, so
   six hundred nodes cost nothing to paint. */
.node-shade {
  fill: rgb(0 0 0 / 0.42);
}

.node-body {
  fill: color-mix(in srgb, var(--tint) 7%, var(--color-surface-raised));
  stroke: color-mix(in srgb, var(--tint) 40%, transparent);
  stroke-width: 1.1;
  stroke-linejoin: round;
  transition:
    stroke 140ms ease,
    fill 140ms ease;
}

.node-detail {
  fill: none;
  stroke: color-mix(in srgb, var(--tint) 30%, transparent);
  stroke-width: 1;
}

.node-stack {
  fill: var(--color-surface);
  stroke: color-mix(in srgb, var(--tint) 26%, transparent);
  stroke-width: 1;
}

.node-halo {
  fill: none;
  stroke: transparent;
  stroke-width: 7;
  stroke-linejoin: round;
  transition: stroke 160ms ease;
}

.node-socket {
  fill: color-mix(in srgb, var(--tint) 13%, var(--color-canvas));
  stroke: color-mix(in srgb, var(--tint) 34%, transparent);
  stroke-width: 1;
}

.node-glyph {
  color: var(--tint);
}

.node-kind {
  font-family: var(--font-mono);
  font-size: 9.5px;
  fill: color-mix(in srgb, var(--tint) 80%, var(--color-text-muted));
}

.node-title {
  font-family: var(--font-sans);
  font-size: 12.5px;
  font-weight: 500;
  fill: var(--color-text);
}

.is-code .node-body {
  fill: color-mix(in srgb, var(--tint) 4%, var(--color-canvas));
}

.is-code .node-title {
  font-family: var(--font-mono);
  font-size: 12px;
}

.node-fold rect {
  fill: color-mix(in srgb, var(--tint) 18%, var(--color-canvas));
  stroke: color-mix(in srgb, var(--tint) 45%, transparent);
}

.node-fold text {
  font-family: var(--font-mono);
  font-size: 10px;
  fill: var(--color-text);
}

.diagram-node:hover .node-body {
  stroke: color-mix(in srgb, var(--tint) 75%, transparent);
}

.diagram-node:focus-visible .node-halo {
  stroke: color-mix(in srgb, var(--color-accent) 55%, transparent);
}

/* Selected: the node is where you are, so it takes the accent, with a soft halo around its
   own outline rather than a box drawn over it. */
.is-selected .node-body {
  stroke: var(--color-accent);
  stroke-width: 1.5;
  fill: color-mix(in srgb, var(--color-accent) 7%, var(--color-surface-raised));
}

.is-selected .node-halo {
  stroke: color-mix(in srgb, var(--color-accent) 18%, transparent);
}

.is-lit .node-body {
  stroke: color-mix(in srgb, var(--tint) 80%, transparent);
}

.is-dimmed {
  opacity: 0.22;
}
</style>
