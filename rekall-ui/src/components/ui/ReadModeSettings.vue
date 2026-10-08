<script setup lang="ts">
import { ref } from 'vue'
import { READ_WIDTH_MAX, READ_WIDTH_MIN, type ReadAlign } from '@/common/config/read-mode'

/** The `Aa` button over a page being read, and the reading column's width and alignment it sets. */
const width = defineModel<number>('width', { required: true })
const align = defineModel<ReadAlign>('align', { required: true })

const ALIGNMENTS = ['left', 'center'] as const
const open = ref(false)
</script>

<template>
  <div class="relative">
    <button
      type="button"
      class="focus-ring flex size-7 items-center justify-center rounded-full border border-border bg-surface text-caption font-semibold text-text-subtle shadow-sm transition-colors hover:text-text"
      :aria-expanded="open"
      aria-label="Reading settings"
      data-testid="read-settings-toggle"
      @click="open = !open"
    >
      Aa
    </button>
    <div
      v-if="open"
      class="absolute right-0 top-9 z-10 w-56 rounded-card border border-border bg-surface-raised p-3 shadow-lg"
      data-testid="read-settings-panel"
    >
      <div class="flex items-center justify-between gap-2">
        <span class="text-caption font-medium text-text-subtle">Line width</span>
        <span class="text-caption tabular-nums text-text-subtle">{{ width }}px</span>
      </div>
      <input
        v-model.number="width"
        type="range"
        class="mt-1.5 w-full accent-accent"
        :min="READ_WIDTH_MIN"
        :max="READ_WIDTH_MAX"
        step="20"
        aria-label="Reading line width"
        data-testid="read-width-slider"
      />

      <p class="mt-3 text-caption font-medium text-text-subtle">Align</p>
      <div class="mt-1.5 flex gap-0.5 rounded-control bg-canvas p-0.5">
        <button
          v-for="option in ALIGNMENTS"
          :key="option"
          type="button"
          class="focus-ring h-6 flex-1 rounded-control text-caption capitalize transition-colors"
          :class="align === option ? 'bg-surface-raised text-text' : 'text-text-subtle hover:text-text'"
          :aria-pressed="align === option"
          :data-testid="`read-align-${option}`"
          @click="align = option"
        >
          {{ option }}
        </button>
      </div>
    </div>
  </div>
</template>
