<script setup lang="ts">
import { computed } from 'vue'
import { messageKey, type SequenceMessage } from '@/common/diagram/scene/sequence-scene'
import { textWidth } from '@/common/diagram/text-wrap'
import type { Emphasis } from '@/common/diagram/emphasis'
import type { PlacedMessage } from '@/common/diagram/layout/sequence-layout'

/**
 * One message: a solid call or a dashed reply, with the head its arrow names, the text above it
 * and its number when the sheet numbers them. The whole row is the target: thin lines are hard
 * to hit.
 */
const props = defineProps<{ message: SequenceMessage; placed: PlacedMessage; numbered: boolean; linked: boolean; emphasis: Emphasis }>()

const emit = defineEmits<{ select: [key: string] }>()

const LOOP_WIDTH = 34
const LOOP_HEIGHT = 26
const LABEL_FONT = 11.5

const key = computed(() => messageKey(props.message.number))
const direction = computed(() => (props.placed.toX >= props.placed.fromX ? 1 : -1))
const line = computed(() => {
  const { fromX, toX, y, self } = props.placed
  if (self) return `M${fromX},${y}H${fromX + LOOP_WIDTH}V${y + LOOP_HEIGHT}H${fromX + 2}`
  return `M${fromX},${y}H${toX - direction.value * 2}`
})
const tip = computed(() =>
  props.placed.self ? { x: props.placed.fromX, y: props.placed.y + LOOP_HEIGHT, angle: 180 } : { x: props.placed.toX, y: props.placed.y, angle: direction.value > 0 ? 0 : 180 }
)
const hitBox = computed(() => {
  const { fromX, toX, y, self } = props.placed
  const left = self ? fromX : Math.min(fromX, toX)
  const width = self ? LOOP_WIDTH + 12 + textWidth(props.message.text, LABEL_FONT) : Math.abs(toX - fromX)
  return { x: left, y: y - 22, width: Math.max(width, 24), height: self ? LOOP_HEIGHT + 30 : 32 }
})
const label = computed(() => `Message ${props.message.number}: ${props.message.text}${props.linked ? ', opens its code' : ''}`)

function onKeydown(event: KeyboardEvent): void {
  if (event.key !== 'Enter' && event.key !== ' ') return
  event.preventDefault()
  emit('select', key.value)
}
</script>

<template>
  <g
    class="sequence-message"
    :class="[`line-${message.line}`, `is-${emphasis}`]"
    role="button"
    tabindex="0"
    :aria-label="label"
    :aria-pressed="emphasis === 'selected'"
    :data-element="key"
    @click.stop="emit('select', key)"
    @keydown="onKeydown"
  >
    <rect :x="hitBox.x" :y="hitBox.y" :width="hitBox.width" :height="hitBox.height" class="message-hit" />
    <path :d="line" class="message-line" />
    <g :transform="`translate(${tip.x} ${tip.y}) rotate(${tip.angle})`" class="message-head">
      <path v-if="message.head === 'arrow' || message.head === 'both'" d="M0,0L-8.5,-4.4L-6.4,0L-8.5,4.4Z" class="head-filled" />
      <path v-else-if="message.head === 'async'" d="M-8,-4.5L0,0L-8,4.5" class="head-open" />
      <path v-else-if="message.head === 'cross'" d="M-9,-4.5L-1,4.5M-9,4.5L-1,-4.5" class="head-open" />
    </g>
    <path
      v-if="message.head === 'both' && !placed.self"
      d="M0,0L-8.5,-4.4L-6.4,0L-8.5,4.4Z"
      class="head-filled"
      :transform="`translate(${placed.fromX} ${placed.y}) rotate(${direction > 0 ? 180 : 0})`"
    />
    <text :x="placed.labelAt.x" :y="placed.labelAt.y" :text-anchor="placed.self ? 'start' : 'middle'" class="message-text">
      {{ message.text }}<tspan v-if="linked" class="message-code" dx="5">&lt;/&gt;</tspan>
    </text>
    <g v-if="numbered" :transform="`translate(${placed.fromX} ${placed.y})`" class="message-number">
      <circle r="8.5" />
      <text text-anchor="middle" y="3.4">{{ message.number }}</text>
    </g>
  </g>
</template>

<style scoped>
.sequence-message {
  --line-tint: var(--color-text-muted);
  cursor: pointer;
  outline: none;
  transition: opacity var(--motion-quick) ease;
}

.message-hit {
  fill: transparent;
  rx: 6;
  transition: fill var(--motion-quick) ease;
}

.message-line {
  fill: none;
  stroke: var(--line-tint);
  stroke-width: 1.4;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.line-dashed .message-line {
  stroke-dasharray: 5 4;
}

.head-filled {
  fill: var(--line-tint);
}

.head-open {
  fill: none;
  stroke: var(--line-tint);
  stroke-width: 1.4;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.message-text {
  paint-order: stroke;
  stroke: var(--color-canvas);
  stroke-width: 4px;
  stroke-linejoin: round;
  font-family: var(--font-sans);
  font-size: var(--text-caption);
  fill: var(--color-text);
}

.line-dashed .message-text {
  fill: var(--color-text-muted);
}

.message-code {
  font-family: var(--font-mono);
  font-size: var(--text-micro);
  fill: var(--color-kind-action);
}

.message-number circle {
  fill: var(--color-surface-raised);
  stroke: var(--color-border-strong);
}

.message-number text {
  font-family: var(--font-mono);
  font-size: var(--text-micro);
  fill: var(--color-text-muted);
}

.sequence-message:hover .message-hit,
.sequence-message:focus-visible .message-hit {
  fill: color-mix(in srgb, var(--color-text) 5%, transparent);
}

.sequence-message:focus-visible .message-hit {
  stroke: color-mix(in srgb, var(--color-accent) 70%, transparent);
  stroke-width: 1.5;
}

.is-selected {
  --line-tint: var(--color-accent);
}

.is-selected .message-hit {
  fill: color-mix(in srgb, var(--color-accent) 9%, transparent);
}

.is-lit {
  --line-tint: var(--color-accent-strong);
}

.is-dimmed {
  opacity: 0.18;
}
</style>
