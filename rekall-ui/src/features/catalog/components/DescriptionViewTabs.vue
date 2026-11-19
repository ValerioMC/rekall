<script setup lang="ts">
import type { DescriptionView } from '@/features/catalog/model/description-view'

/**
 * Switches the description window between the text and its revisions. The revisions tab carries how
 * many there are, and an accent dot while one waits for a run; the dot pings while a session is on one.
 */
defineProps<{ modelValue: DescriptionView; count: number; attention: boolean; running: boolean }>()
const emit = defineEmits<{ 'update:modelValue': [view: DescriptionView] }>()
</script>

<template>
  <div class="flex shrink-0 gap-4" role="group" aria-label="View">
    <button
      type="button"
      class="focus-ring -mb-px flex h-8 items-center border-b-2 text-caption transition-colors"
      :class="modelValue === 'description' ? 'border-accent text-text' : 'border-transparent text-text-subtle hover:text-text'"
      :aria-pressed="modelValue === 'description'"
      data-testid="view-description"
      @click="emit('update:modelValue', 'description')"
    >
      Description
    </button>
    <button
      type="button"
      class="focus-ring -mb-px flex h-8 items-center gap-1.5 border-b-2 text-caption transition-colors"
      :class="modelValue === 'revisions' ? 'border-accent text-text' : 'border-transparent text-text-subtle hover:text-text'"
      :aria-pressed="modelValue === 'revisions'"
      data-testid="view-revisions"
      @click="emit('update:modelValue', 'revisions')"
    >
      Revisions
      <span v-if="count > 0" class="text-micro text-text-muted" data-testid="view-revisions-count">{{ count }}</span>
      <span
        v-if="attention"
        class="relative grid size-2 place-items-center"
        :aria-label="running ? 'A revision is running' : 'Needs a run'"
        data-testid="view-revisions-attention"
      >
        <span v-if="running" class="absolute inline-flex size-2 animate-ping rounded-full bg-accent/60 motion-reduce:animate-none" data-testid="view-revisions-running" />
        <span class="relative inline-flex size-1.5 rounded-full bg-accent" />
      </span>
    </button>
  </div>
</template>
