<script setup lang="ts">
import { computed } from 'vue'
import ElementGlyph from '@/features/diagram/components/canvas/ElementGlyph.vue'
import { detailPath, shapePath } from '@/features/diagram/lib/node-shape'
import type { ElementStyle } from '@/features/diagram/lib/element-style'
import type { Emphasis } from '@/features/diagram/lib/emphasis'
import type { NodeOutline } from '@/features/diagram/lib/scene/graph-scene'

/**
 * The card every box on a canvas is drawn as: the outline carries the shape, the socket on the
 * left the kind, and a code mark on the right says a click opens the code behind it.
 */
const props = defineProps<{
  elementKey: string
  x: number
  y: number
  width: number
  height: number
  outline: NodeOutline
  look: ElementStyle
  lines: readonly string[]
  linked: boolean
  emphasis: Emphasis
  arriveDelay?: number
  /** A box with rows keeps its socket and title in this band at the top; the slot draws the rest. */
  headerHeight?: number
}>()

const emit = defineEmits<{ select: [key: string] }>()

const outlinePath = computed(() => shapePath(props.outline, props.width, props.height))
const detail = computed(() => detailPath(props.outline, props.width, props.height))
const band = computed(() => props.headerHeight ?? props.height)
const textTop = computed(() => (band.value - (15 + props.lines.length * 16)) / 2 + 1)
const socketY = computed(() => band.value / 2)
const label = computed(() => `${props.look.label}: ${props.lines.join(' ')}${props.linked ? ', opens its code' : ''}`)

function onKeydown(event: KeyboardEvent): void {
  if (event.key !== 'Enter' && event.key !== ' ') return
  event.preventDefault()
  emit('select', props.elementKey)
}
</script>

<template>
  <g
    class="element-card"
    :class="`is-${emphasis}`"
    :style="{ '--tint': look.tint, transform: `translate(${x}px, ${y}px)`, '--arrive-delay': `${arriveDelay ?? 0}ms` }"
    role="button"
    tabindex="0"
    :aria-label="label"
    :aria-pressed="emphasis === 'selected'"
    :data-element="elementKey"
    @click.stop="emit('select', elementKey)"
    @keydown="onKeydown"
  >
    <g class="card-arrive">
      <path :d="outlinePath" class="card-halo" />
      <path :d="outlinePath" class="card-shade" transform="translate(0 3)" />
      <path :d="outlinePath" class="card-body" />
      <path v-if="detail" :d="detail" class="card-detail" />

      <circle cx="24" :cy="socketY" r="13" class="card-socket" />
      <ElementGlyph :glyph="look.glyph" :x="16" :y="socketY - 8" class="card-glyph" />

      <text :x="46" :y="textTop + 9" class="card-kind">{{ look.label.toLowerCase() }}</text>
      <text v-for="(line, index) in lines" :key="index" :x="46" :y="textTop + 27 + index * 16" class="card-title">{{ line }}</text>

      <slot />

      <g v-if="linked" class="card-link" :transform="`translate(${width - 26} 8)`">
        <rect width="18" height="14" rx="4" />
        <path d="M7 4.6 4.6 7 7 9.4M11 4.6 13.4 7 11 9.4" />
      </g>
    </g>
  </g>
</template>

<style scoped>
.element-card {
  cursor: pointer;
  outline: none;
  transition:
    transform var(--motion-slow) cubic-bezier(0.16, 1, 0.3, 1),
    opacity var(--motion-quick) ease;
}

.card-arrive {
  animation: card-arrive 320ms cubic-bezier(0.16, 1, 0.3, 1) both;
  animation-delay: var(--arrive-delay);
  transform-box: fill-box;
  transform-origin: center;
}

@keyframes card-arrive {
  from {
    opacity: 0;
    transform: translateY(6px) scale(0.97);
  }
}

@media (prefers-reduced-motion: reduce) {
  .card-arrive {
    animation: none;
  }
}

/* Recessed like a slot in the canvas rather than floating: a hard shade 3px down, no blur, so
   hundreds of cards cost nothing to paint. */
.card-shade {
  fill: rgb(0 0 0 / 0.42);
}

.card-body {
  fill: color-mix(in srgb, var(--tint) 7%, var(--color-surface-raised));
  stroke: color-mix(in srgb, var(--tint) 40%, transparent);
  stroke-width: 1.1;
  stroke-linejoin: round;
  transition:
    stroke var(--motion-quick) ease,
    fill var(--motion-quick) ease;
}

.card-detail {
  fill: none;
  stroke: color-mix(in srgb, var(--tint) 30%, transparent);
  stroke-width: 1;
}

.card-halo {
  fill: none;
  stroke: transparent;
  stroke-width: 7;
  stroke-linejoin: round;
  transition: stroke var(--motion-quick) ease;
}

.card-socket {
  fill: color-mix(in srgb, var(--tint) 13%, var(--color-canvas));
  stroke: color-mix(in srgb, var(--tint) 34%, transparent);
  stroke-width: 1;
}

.card-glyph {
  color: var(--tint);
}

.card-kind {
  font-family: var(--font-mono);
  font-size: var(--text-micro);
  fill: color-mix(in srgb, var(--tint) 80%, var(--color-text-muted));
}

.card-title {
  font-family: var(--font-sans);
  font-size: var(--text-small);
  font-weight: 500;
  fill: var(--color-text);
}

.card-link rect {
  fill: color-mix(in srgb, var(--tint) 14%, var(--color-canvas));
  stroke: color-mix(in srgb, var(--tint) 38%, transparent);
  stroke-width: 1;
}

.card-link path {
  fill: none;
  stroke: var(--tint);
  stroke-width: 1.3;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.element-card:hover .card-body {
  stroke: color-mix(in srgb, var(--tint) 75%, transparent);
}

.element-card:focus-visible .card-halo {
  stroke: color-mix(in srgb, var(--color-accent) 70%, transparent);
}

/* Selected: the card is where you are, so it takes the accent, with a soft halo around its own
   outline rather than a box drawn over it. */
.is-selected .card-body {
  stroke: var(--color-accent);
  stroke-width: 1.5;
  fill: color-mix(in srgb, var(--color-accent) 7%, var(--color-surface-raised));
}

.is-selected .card-halo {
  stroke: color-mix(in srgb, var(--color-accent) 18%, transparent);
}

.is-lit .card-body {
  stroke: color-mix(in srgb, var(--tint) 80%, transparent);
}

.is-dimmed {
  opacity: 0.22;
}
</style>
