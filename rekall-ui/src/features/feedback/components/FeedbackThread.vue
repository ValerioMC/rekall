<script setup lang="ts">
import FeedbackEntry from '@/features/feedback/components/FeedbackEntry.vue'
import type { Feedback } from '@/features/feedback/model/feedback'

/** The messages under the description, oldest first; the kind chip on each does the telling apart. */
defineProps<{ entries: readonly Feedback[] }>()
const emit = defineEmits<{
  edit: [entry: Feedback, bodyMarkdown: string]
  remove: [entry: Feedback]
  discard: [entry: Feedback]
}>()
</script>

<template>
  <ol v-if="entries.length > 0" class="flex flex-col gap-2" aria-label="Feedback thread" data-testid="feedback-thread">
    <FeedbackEntry
      v-for="entry in entries"
      :key="entry.id"
      :entry="entry"
      @edit="(target, body) => emit('edit', target, body)"
      @remove="emit('remove', $event)"
      @discard="emit('discard', $event)"
    />
  </ol>
  <p v-else class="text-small text-text-subtle" data-testid="feedback-empty">
    Nothing here yet. Rewrite the description or add feedback below, and a session reads each entry once.
  </p>
</template>
