<script setup lang="ts">
/**
 * The seam between two steps of one session. It shows nothing until the pointer or the keyboard
 * is on it, then offers to cut the session there: the steps from here on start a fresh one.
 */
defineProps<{ title: string }>()

defineEmits<{ split: [] }>()
</script>

<template>
  <li class="group/gap relative h-1.5" role="presentation" data-testid="session-gap">
    <slot name="thread" />
    <button
      type="button"
      class="focus-ring absolute -top-2.5 right-2 left-18 z-20 flex h-6 cursor-pointer items-center gap-2.5 text-caption font-medium text-session-strong opacity-0 transition-opacity duration-(--motion-quick) group-hover/gap:opacity-100 focus-visible:opacity-100"
      :aria-label="`Start a new session at ${title}`"
      data-testid="step-new-session"
      @click="$emit('split')"
    >
      <span class="w-3 shrink-0 border-t border-dashed border-session" aria-hidden="true" />
      <span class="inline-flex items-center gap-1.5 bg-canvas px-0.5">
        <svg class="size-2.5" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <path d="M6 2.5v7M2.5 6h7" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
        </svg>
        New session from here
      </span>
      <span class="flex-1 border-t border-dashed border-session/40" aria-hidden="true" />
    </button>
  </li>
</template>
