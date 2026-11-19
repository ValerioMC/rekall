<script setup lang="ts">
import { kindLabel, TARGETS } from '@/features/feedback/lib/feedback-labels'
import type { FeedbackKind } from '@/features/feedback/model/feedback'

defineProps<{ modelValue: FeedbackKind }>()
const emit = defineEmits<{ 'update:modelValue': [kind: FeedbackKind] }>()
</script>

<template>
  <div class="flex gap-0.5 rounded-control bg-canvas p-0.5" role="group" aria-label="What the revision is applied to">
    <button
      v-for="target in TARGETS"
      :key="target"
      type="button"
      class="focus-ring h-6 rounded-control px-2.5 text-caption transition-colors"
      :class="modelValue === target ? 'bg-surface-raised text-text' : 'text-text-subtle hover:text-text'"
      :aria-pressed="modelValue === target"
      :data-testid="`revision-target-${target}`"
      @click="emit('update:modelValue', target)"
    >
      {{ kindLabel(target) }}
    </button>
  </div>
</template>
