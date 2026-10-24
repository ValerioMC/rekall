<script setup lang="ts">
import ConnectionKindIcon from '@/features/connection/components/ConnectionKindIcon.vue'
import { STEP_PRESETS, type StepPreset } from '@/features/graph/lib/graph-presets'

/** The kinds of step a graph is built from, each added in one click after the picked step. */
defineProps<{ disabled: boolean }>()

const emit = defineEmits<{ add: [preset: StepPreset] }>()
</script>

<template>
  <div class="flex flex-wrap items-center gap-1.5" role="toolbar" aria-label="Add a step" data-testid="graph-palette">
    <span class="eyebrow mr-1">Add</span>
    <button
      v-for="preset in STEP_PRESETS"
      :key="preset.id"
      type="button"
      class="key-quiet focus-ring inline-flex h-7 cursor-pointer items-center gap-1.5 rounded-control px-2.5 text-caption disabled:cursor-not-allowed disabled:opacity-40"
      :disabled="disabled"
      :data-testid="`graph-add-${preset.id}`"
      @click="emit('add', preset)"
    >
      <ConnectionKindIcon v-if="preset.kind" :kind="preset.kind" :size="12" />
      <svg v-else class="size-3" viewBox="0 0 14 14" fill="none" aria-hidden="true">
        <path d="M7 2.5v9M2.5 7h9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
      </svg>
      {{ preset.label }}
    </button>
  </div>
</template>
