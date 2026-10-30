<script setup lang="ts">
import { computed, ref } from 'vue'
import { splitLines } from '@/features/chat/lib/lines'

/**
 * What a tool printed, on the dark well: the last `limit` lines, the rest behind a key, since the
 * end of an output is where its result is. `truncated` says Rekall cut it before it got here.
 */
const props = withDefaults(
  defineProps<{ text: string; truncated: boolean; isError: boolean; limit?: number }>(),
  { limit: 40 }
)

const expanded = ref(false)

const lines = computed(() => splitLines(props.text))
const hidden = computed(() => Math.max(0, lines.value.length - props.limit))
const shown = computed(() => (expanded.value ? lines.value : lines.value.slice(hidden.value)))
</script>

<template>
  <div class="overflow-hidden rounded-control border border-border bg-(--color-well-top)" data-testid="tool-output">
    <button
      v-if="hidden > 0 && !expanded"
      type="button"
      class="focus-ring w-full cursor-pointer border-b border-border bg-surface px-2 py-1 text-left text-caption text-text-muted hover:text-text"
      @click="expanded = true"
    >
      Show all {{ lines.length }} lines
    </button>
    <pre
      class="max-h-80 overflow-auto px-2 py-1.5 font-mono text-small whitespace-pre-wrap break-words"
      :class="isError ? 'text-danger' : 'text-text-muted'"
      tabindex="0"
      >{{ shown.join('\n') }}</pre
    >
    <p v-if="truncated" class="border-t border-border px-2 py-1 text-caption text-text-subtle">output cut by Rekall</p>
  </div>
</template>
