<script setup lang="ts">
import { computed } from 'vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import { useStreamedText } from '@/features/chat/composables/useStreamedText'

/**
 * One message of the conversation. The person's is plain text, with a slash command in mono; the
 * assistant's is markdown through the renderer the wrapup uses, repainted a few times a second
 * while it arrives and announced once, when it is complete.
 */
const props = defineProps<{ role: 'user' | 'assistant'; text: string; complete: boolean }>()

const shown = useStreamedText(
  () => props.text,
  () => props.complete
)
const isCommand = computed(() => props.role === 'user' && props.text.startsWith('/'))
</script>

<template>
  <div v-if="role === 'user'" class="flex justify-end" data-testid="chat-message" data-role="user">
    <div class="max-w-[85%] rounded-card bg-surface-raised px-3 py-2">
      <p v-if="isCommand" class="mb-0.5 text-micro font-semibold tracking-wide text-text-subtle uppercase">command</p>
      <p class="text-body whitespace-pre-wrap break-words text-text" :class="isCommand && 'font-mono text-small'">{{ text }}</p>
    </div>
  </div>

  <div
    v-else
    class="max-w-[65ch]"
    data-testid="chat-message"
    data-role="assistant"
    aria-live="polite"
    :aria-busy="!complete"
  >
    <AppMarkdownEditor :model-value="shown" readonly bare />
    <span v-if="!complete" class="chat-caret" aria-hidden="true" data-testid="chat-caret" />
  </div>
</template>

<style scoped>
.chat-caret {
  display: inline-block;
  width: 2px;
  height: 1em;
  margin-left: 2px;
  vertical-align: text-bottom;
  background: var(--color-accent);
  animation: chat-caret-blink 1s steps(2, start) infinite;
}

@keyframes chat-caret-blink {
  to {
    visibility: hidden;
  }
}

@media (prefers-reduced-motion: reduce) {
  .chat-caret {
    animation: none;
  }
}
</style>
