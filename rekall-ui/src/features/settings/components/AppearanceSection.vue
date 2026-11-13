<script setup lang="ts">
import { storeToRefs } from 'pinia'
import SegmentedControl, { type SegmentOption } from '@/components/ui/SegmentedControl.vue'
import { usePreferencesStore } from '@/features/settings/store/preferences.store'
import { useWorkStore } from '@/features/work/store/work.store'
import { SCALES, type Motion } from '@/common/config/ui-preferences'
import type { WorkDensity } from '@/features/work/model/work-view'

const preferences = usePreferencesStore()
const work = useWorkStore()
const { motion, scale } = storeToRefs(preferences)
const { density } = storeToRefs(work)

const DENSITY: readonly SegmentOption<WorkDensity>[] = [
  { value: 'comfortable', label: 'Comfortable' },
  { value: 'compact', label: 'Compact' }
]
const MOTION: readonly SegmentOption<Motion>[] = [
  { value: 'full', label: 'Full' },
  { value: 'reduced', label: 'Reduced' }
]
const SCALE_OPTIONS: readonly SegmentOption<string>[] = SCALES.map((value) => ({ value: String(value), label: `${value}%` }))

function chooseScale(value: string): void {
  scale.value = SCALES.find((candidate) => String(candidate) === value) ?? scale.value
}
</script>

<template>
  <section class="flex flex-col gap-6" aria-label="Appearance" data-testid="appearance-section">
    <h2 class="text-heading font-medium text-text">Appearance</h2>

    <div class="flex flex-col gap-2">
      <p class="text-body font-medium text-text">Density</p>
      <p class="text-small text-text-muted">How tall a row of the Work ledger is.</p>
      <SegmentedControl v-model="density" :options="DENSITY" label="Density" />
    </div>

    <div class="flex flex-col gap-2">
      <p class="text-body font-medium text-text">Motion</p>
      <p class="text-small text-text-muted">Reduced stops the loops, the running comet, the tube sheen and the session pulse. Every state change still shows.</p>
      <SegmentedControl v-model="motion" :options="MOTION" label="Motion" />
    </div>

    <div class="flex flex-col gap-2">
      <p class="text-body font-medium text-text">Text scale</p>
      <p class="text-small text-text-muted">Scales every size in the console.</p>
      <SegmentedControl :model-value="String(scale)" :options="SCALE_OPTIONS" label="Text scale" @update:model-value="chooseScale" />
    </div>
  </section>
</template>
