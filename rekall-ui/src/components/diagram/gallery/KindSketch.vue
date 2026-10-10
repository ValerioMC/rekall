<script setup lang="ts">
import type { DiagramKind } from '@/model/diagram'

/**
 * The picture on a gallery card: a quiet sketch of what that type of diagram looks like, in the
 * canvas' own tints. Decorative; the card's text says what it is.
 */
defineProps<{ kind: DiagramKind }>()
</script>

<template>
  <svg viewBox="0 0 240 88" class="h-full w-full" fill="none" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <template v-if="kind === 'sequence'">
      <g v-for="x in [48, 120, 192]" :key="x">
        <rect :x="x - 26" y="8" width="52" height="16" rx="5" class="sketch-card" />
        <path :d="`M${x},24V84`" class="sketch-life" />
      </g>
      <rect x="117" y="38" width="6" height="34" rx="2" class="sketch-bar" />
      <path d="M48,40H114M108,36l6,4-6,4" class="sketch-line" />
      <path d="M123,52H186M180,48l6,4-6,4" class="sketch-line" />
      <path d="M186,64H126M132,60l-6,4 6,4" class="sketch-line is-dashed" />
    </template>
    <template v-else-if="kind === 'flowchart'">
      <rect x="20" y="34" width="52" height="20" rx="5" class="sketch-card" />
      <path d="M120,28L140,44L120,60L100,44Z" class="sketch-card is-decision" />
      <rect x="168" y="12" width="52" height="20" rx="5" class="sketch-card" />
      <rect x="168" y="56" width="52" height="20" rx="10" class="sketch-card is-terminal" />
      <path d="M72,44H100M140,44C154,44 154,22 168,22M140,44C154,44 154,66 168,66" class="sketch-line" />
    </template>
    <template v-else-if="kind === 'state'">
      <circle cx="22" cy="44" r="6" class="sketch-dot" />
      <rect x="44" y="32" width="56" height="24" rx="12" class="sketch-card is-state" />
      <rect x="132" y="12" width="56" height="24" rx="12" class="sketch-card is-state" />
      <rect x="132" y="54" width="56" height="24" rx="12" class="sketch-card is-state" />
      <circle cx="218" cy="24" r="7" class="sketch-ring" />
      <path d="M28,44H44M100,44C116,44 116,24 132,24M100,44C116,44 116,66 132,66M188,24H211" class="sketch-line" />
    </template>
    <template v-else-if="kind === 'class' || kind === 'er'">
      <g v-for="box in [{ x: 24, y: 10 }, { x: 140, y: 10 }]" :key="box.x">
        <rect :x="box.x" :y="box.y" width="76" height="68" rx="6" class="sketch-card" :class="kind === 'er' ? 'is-entity' : 'is-class'" />
        <path :d="`M${box.x},${box.y + 18}H${box.x + 76}`" class="sketch-rule" />
        <path v-for="row in [30, 42, 54]" :key="row" :d="`M${box.x + 10},${box.y + row}H${box.x + 54}`" class="sketch-text" />
      </g>
      <path d="M100,44H140" class="sketch-line" />
      <path v-if="kind === 'class'" d="M100,44L110,38V50Z" class="sketch-open" />
      <path v-else d="M134,38L140,44L134,50M134,44H140M106,38V50" class="sketch-line" />
    </template>
    <template v-else>
      <rect x="24" y="18" width="192" height="52" rx="10" class="sketch-card is-native" />
      <path d="M48,56V44M72,56V34M96,56V48M120,56V30M144,56V40M168,56V36M192,56V46" class="sketch-line" />
    </template>
  </svg>
</template>

<style scoped>
.sketch-card {
  --tint: var(--color-kind-action);
  fill: color-mix(in srgb, var(--tint) 10%, var(--color-surface-raised));
  stroke: color-mix(in srgb, var(--tint) 45%, transparent);
  stroke-width: 1.2;
}

.is-decision {
  --tint: var(--color-kind-decision);
}

.is-terminal {
  --tint: var(--color-kind-event);
}

.is-state {
  --tint: var(--color-kind-state);
}

.is-class {
  --tint: var(--color-kind-concept);
}

.is-entity {
  --tint: var(--color-kind-data);
}

.is-native {
  --tint: var(--color-kind-external);
}

.sketch-life {
  stroke: var(--color-border-strong);
  stroke-dasharray: 2 3;
}

.sketch-bar {
  fill: color-mix(in srgb, var(--color-kind-action) 25%, var(--color-surface-raised));
  stroke: color-mix(in srgb, var(--color-kind-action) 45%, transparent);
}

.sketch-line {
  stroke: var(--color-text-subtle);
  stroke-width: 1.3;
}

.is-dashed {
  stroke-dasharray: 4 3;
}

.sketch-rule {
  stroke: color-mix(in srgb, var(--color-text-subtle) 40%, transparent);
}

.sketch-text {
  stroke: color-mix(in srgb, var(--color-text-subtle) 55%, transparent);
  stroke-width: 2.2;
}

.sketch-open {
  fill: var(--color-canvas);
  stroke: var(--color-text-subtle);
}

.sketch-dot {
  fill: var(--color-kind-state);
}

.sketch-ring {
  stroke: var(--color-kind-state);
  stroke-width: 1.4;
}
</style>
