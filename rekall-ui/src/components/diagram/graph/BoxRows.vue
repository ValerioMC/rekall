<script setup lang="ts">
import { computed } from 'vue'
import { BOX_HEADER, COMPARTMENT_PADDING, ROW_HEIGHT } from '@/common/diagram/layout/node-size'
import type { CompartmentRow } from '@/common/diagram/scene/graph-scene'

/** A class's members and methods, or an entity's attributes, in compartments under its header. */
const props = defineProps<{ width: number; compartments: readonly (readonly CompartmentRow[])[] }>()

interface PlacedRow {
  readonly key: string
  readonly y: number
  readonly row: CompartmentRow
}

const layout = computed(() => {
  const dividers: number[] = []
  const rows: PlacedRow[] = []
  let y = BOX_HEADER
  props.compartments
    .filter((compartment) => compartment.length > 0)
    .forEach((compartment, at) => {
      dividers.push(y)
      y += COMPARTMENT_PADDING
      compartment.forEach((row, index) => rows.push({ key: `${at}-${index}`, y: y + ROW_HEIGHT * index + 13, row }))
      y += compartment.length * ROW_HEIGHT + COMPARTMENT_PADDING
    })
  return { dividers, rows }
})
</script>

<template>
  <g class="box-rows">
    <path v-for="y in layout.dividers" :key="y" :d="`M1,${y}H${width - 1}`" class="box-divider" />
    <template v-for="placed in layout.rows" :key="placed.key">
      <text x="14" :y="placed.y" class="box-row">{{ placed.row.text }}</text>
      <text v-if="placed.row.detail" :x="width - 14" :y="placed.y" text-anchor="end" class="box-detail">{{ placed.row.detail }}</text>
    </template>
  </g>
</template>

<style scoped>
.box-divider {
  stroke: color-mix(in srgb, var(--tint) 26%, transparent);
  stroke-width: 1;
}

.box-row {
  font-family: var(--font-mono);
  font-size: var(--text-micro);
  fill: var(--color-text);
}

.box-detail {
  font-family: var(--font-mono);
  font-size: var(--text-micro);
  fill: var(--color-text-subtle);
}
</style>
