<script setup lang="ts">
import { computed } from 'vue'

/**
 * The end of a session: why it ended, and whether the next session of the task is on its way.
 * A session that ended with a non-zero exit and nothing after it is shown as an error.
 */
const props = defineProps<{ exitCode: number; detail: string; followingNext: boolean }>()

const failed = computed(() => props.exitCode !== 0)
</script>

<template>
  <div
    class="rounded-card border px-3 py-2 text-small"
    :class="failed && !followingNext ? 'border-danger/50 bg-danger-soft text-text' : 'border-border bg-surface text-text-muted'"
    role="status"
    data-testid="chat-ended"
  >
    <p class="font-semibold" :class="failed && !followingNext ? 'text-danger' : 'text-text'">Session ended</p>
    <p v-if="detail || failed" class="mt-0.5 font-mono text-caption">exit {{ exitCode }}<template v-if="detail"> · {{ detail }}</template></p>
    <p v-if="followingNext" class="mt-1 flex items-center gap-2" data-testid="chat-ended-following">
      <svg class="size-3 animate-spin" viewBox="0 0 12 12" fill="none" aria-hidden="true">
        <path d="M10 6a4 4 0 1 1-4-4" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
      </svg>
      Opening the next session…
    </p>
    <p v-else-if="!failed" class="mt-1">Session finished. Start a new one with Run here.</p>
  </div>
</template>
