<script setup lang="ts">
import { computed } from 'vue'
import MemoryTypeChip from './MemoryTypeChip.vue'
import type { ResourceSummary } from '@/features/claude-resources/model/claude-resources'

const props = defineProps<{ summary: ResourceSummary; selected: boolean }>()
defineEmits<{ select: [] }>()

const lockLabel = computed(() => {
  const source = props.summary.source
  if (!source || source.kind === 'own') return 'Read only'
  return `Read only: provided by ${source.name}${source.detail ? ` (${source.detail})` : ''}`
})
</script>

<template>
  <div class="group/row relative">
    <button
      class="focus-ring flex w-full items-start gap-2 rounded-control px-2.5 py-1.5 text-left transition-colors"
      :class="[selected ? 'selected-row text-text' : 'text-text-muted hover:bg-surface-raised hover:text-text', $slots.actions ? 'pr-9' : '']"
      :aria-current="selected ? 'true' : undefined"
      data-testid="resource-row"
      @click="$emit('select')"
    >
      <span class="min-w-0 flex-1">
        <span class="block truncate text-body font-medium leading-[1.35]">{{ summary.name }}</span>
        <span v-if="summary.description" class="mt-0.5 line-clamp-1 text-caption leading-[1.4] text-text-subtle">
          {{ summary.description }}
        </span>
      </span>
      <svg v-if="!summary.editable" class="mt-1 size-3 shrink-0 text-text-subtle" viewBox="0 0 16 16" fill="none" role="img" :aria-label="lockLabel">
        <title>{{ lockLabel }}</title>
        <rect x="3.5" y="7" width="9" height="6.5" rx="1.5" stroke="currentColor" stroke-width="1.3" />
        <path d="M5.5 7V5a2.5 2.5 0 0 1 5 0v2" stroke="currentColor" stroke-width="1.3" />
      </svg>
      <MemoryTypeChip :type="summary.memoryType" class="mt-0.5" />
    </button>
    <div
      v-if="$slots.actions"
      class="absolute right-1 top-1 opacity-0 transition-opacity focus-within:opacity-100 group-hover/row:opacity-100 [&:has([aria-expanded=true])]:opacity-100"
    >
      <slot name="actions" />
    </div>
  </div>
</template>
