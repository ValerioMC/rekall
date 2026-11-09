<script setup lang="ts">
import type { DescriptionView } from '@/features/catalog/model/description-view'

/**
 * Switches the description window between the text and its revisions. The revisions tab carries how
 * many there are, and an accent dot while one waits for a run or a session is on one.
 */
defineProps<{ modelValue: DescriptionView; count: number; attention: boolean }>()
const emit = defineEmits<{ 'update:modelValue': [view: DescriptionView] }>()
</script>

<template>
  <div class="flex shrink-0 gap-0.5 rounded-control bg-surface p-0.5" role="group" aria-label="View">
    <button
      type="button"
      class="focus-ring h-6 rounded-control px-3 text-caption transition-colors"
      :class="modelValue === 'description' ? 'bg-surface-raised text-text' : 'text-text-subtle hover:text-text'"
      :aria-pressed="modelValue === 'description'"
      data-testid="view-description"
      @click="emit('update:modelValue', 'description')"
    >
      Description
    </button>
    <button
      type="button"
      class="focus-ring flex h-6 items-center gap-1.5 rounded-control px-3 text-caption transition-colors"
      :class="modelValue === 'revisions' ? 'bg-surface-raised text-text' : 'text-text-subtle hover:text-text'"
      :aria-pressed="modelValue === 'revisions'"
      data-testid="view-revisions"
      @click="emit('update:modelValue', 'revisions')"
    >
      Revisions
      <span v-if="count > 0" class="text-micro text-text-muted" data-testid="view-revisions-count">{{ count }}</span>
      <span v-if="attention" class="size-1.5 rounded-full bg-accent" data-testid="view-revisions-attention" aria-label="Needs a run" />
    </button>
  </div>
</template>
