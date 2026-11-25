<script setup lang="ts">
import type { DiagramScenario } from '@/features/diagram/model/diagram'

/**
 * The opening of a walkthrough: the case it follows, read as a short story from what someone does
 * to what they see at the end, before the first call is played.
 */
defineProps<{ scenario: DiagramScenario }>()

const emit = defineEmits<{ start: [] }>()
</script>

<template>
  <aside
    class="glass pointer-events-auto flex max-h-full w-[min(420px,calc(100vw-2rem))] flex-col overflow-hidden rounded-card border border-border shadow-lift"
    aria-labelledby="scenario-card-title"
    aria-live="polite"
    data-testid="scenario-card"
  >
    <header class="shrink-0 px-4 pb-1 pt-4">
      <p class="section-label">Before the first call</p>
      <h3 id="scenario-card-title" class="mt-1 text-title font-semibold tracking-[-0.01em] text-text">The scenario</h3>
    </header>

    <ol class="story flex min-h-0 flex-col overflow-y-auto px-4 pb-4 pt-3">
      <li class="story-beat">
        <h4 class="section-label mb-1">What happens</h4>
        <p class="text-body leading-relaxed text-text" data-testid="scenario-situation">{{ scenario.situation }}</p>
      </li>
      <li class="story-beat">
        <h4 class="section-label mb-1">For example</h4>
        <p class="whitespace-pre-wrap break-words rounded-control border border-border bg-canvas px-3 py-2 font-mono text-caption leading-relaxed text-text" data-testid="scenario-example">{{ scenario.example }}</p>
      </li>
      <li class="story-beat is-last">
        <h4 class="section-label mb-1">At the end</h4>
        <p class="text-small leading-relaxed text-text-muted" data-testid="scenario-outcome">{{ scenario.outcome }}</p>
      </li>
    </ol>

    <footer class="shrink-0 border-t border-border px-4 py-3">
      <button class="key-gold focus-ring flex w-full items-center justify-center gap-2 rounded-control px-3 py-2 text-small font-semibold" data-testid="scenario-start" @click="emit('start')">
        Follow it, call by call
        <kbd>→</kbd>
      </button>
    </footer>
  </aside>
</template>

<style scoped>
/* A rail joins the three parts, so they read as one story from start to end. */
.story-beat {
  position: relative;
  padding-left: 20px;
  padding-bottom: 14px;
}

.story-beat::before {
  content: '';
  position: absolute;
  left: 3px;
  top: 5px;
  width: 8px;
  height: 8px;
  border-radius: 999px;
  background: var(--color-accent);
}

.story-beat::after {
  content: '';
  position: absolute;
  left: 6.5px;
  top: 17px;
  bottom: 0;
  width: 1px;
  background: color-mix(in srgb, var(--color-accent) 35%, var(--color-border));
}

.story-beat.is-last {
  padding-bottom: 0;
}

.story-beat.is-last::after {
  display: none;
}
</style>
