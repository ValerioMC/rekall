<script setup lang="ts">
import { computed, ref, useId } from 'vue'

/** The model's reasoning, folded to a line and never opened by itself. */
const props = defineProps<{ text: string; complete: boolean }>()

const PREVIEW_CHARACTERS = 80

const open = ref(false)
const bodyId = `thinking-${useId()}`

const preview = computed(() => {
  const flat = props.text.replace(/\s+/g, ' ').trim()
  return flat.length <= PREVIEW_CHARACTERS ? flat : `${flat.slice(0, PREVIEW_CHARACTERS)}…`
})
</script>

<template>
  <div data-testid="chat-thinking">
    <button
      type="button"
      class="focus-ring flex w-full cursor-pointer items-center gap-2 rounded-control px-1 py-0.5 text-left text-small text-text-muted italic hover:text-text"
      :aria-expanded="open"
      :aria-controls="bodyId"
      @click="open = !open"
    >
      <svg class="size-3 shrink-0 transition-transform duration-(--motion-quick)" :class="open && 'rotate-90'" viewBox="0 0 12 12" fill="none" aria-hidden="true">
        <path d="m4.5 2.5 3.5 3.5-3.5 3.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
      <span class="shrink-0 font-semibold not-italic">Thinking</span>
      <span v-if="!open" class="min-w-0 truncate">{{ preview }}</span>
      <span v-if="!complete" class="sr-only">still thinking</span>
    </button>
    <p
      v-if="open"
      :id="bodyId"
      class="mt-1 border-l border-border-strong pl-3 font-mono text-small whitespace-pre-wrap break-words text-text-muted"
    >
      {{ text }}
    </p>
  </div>
</template>
