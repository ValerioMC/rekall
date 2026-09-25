<script setup lang="ts">
/**
 * The one sheet the dock opens above its bar.
 *
 * Both panels share this chrome so the corner reads as one instrument with two faces rather
 * than two widgets that happen to be neighbours. The header carries the same live mark as the
 * segment that opened it, so the eye lands on the panel already knowing which one it is.
 */
defineProps<{
  panelId: string
  title: string
  count: number
  tint: 'time' | 'session'
}>()

const emit = defineEmits<{ close: [] }>()
</script>

<template>
  <section
    :id="panelId"
    class="dock-panel w-[320px] overflow-hidden rounded-[var(--radius-card)] border border-border-strong bg-surface"
    :class="tint === 'time' ? 'dock-panel-time' : 'dock-panel-session'"
    :aria-label="title"
    data-testid="dock-panel"
  >
    <header class="flex items-center gap-2.5 border-b border-border px-3.5 py-2.5">
      <span v-if="tint === 'time'" class="time-dial shrink-0" aria-hidden="true" />
      <span v-else class="session-caret shrink-0" aria-hidden="true" />
      <h2 class="min-w-0 flex-1 truncate text-[12.5px] font-semibold text-text">
        {{ title }}
      </h2>
      <span
        class="shrink-0 font-mono text-[11px] tabular-nums text-text-subtle"
        data-testid="dock-panel-count"
      >
        {{ count }}
      </span>
      <button
        class="focus-ring grid size-6 shrink-0 place-items-center rounded-full text-text-subtle transition-colors hover:bg-surface-raised hover:text-text"
        aria-label="Close"
        data-testid="dock-panel-close"
        @click="emit('close')"
      >
        <svg class="size-3" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <path d="M3 3l6 6M9 3l-6 6" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
        </svg>
      </button>
    </header>

    <ul class="max-h-[300px] overflow-y-auto p-1.5">
      <slot />
    </ul>
  </section>
</template>
