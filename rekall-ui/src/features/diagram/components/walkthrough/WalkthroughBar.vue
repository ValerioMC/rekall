<script setup lang="ts">
import type { WalkPace } from '@/features/diagram/composables/useWalkthrough'
import type { WalkthroughBeat } from '@/features/diagram/lib/walkthrough/walkthrough-beats'

/**
 * The walkthrough's transport: where the reader is among the calls stepped into, the controls
 * to go back and forth or play, and one tick per beat to jump straight to it.
 */
defineProps<{
  beats: readonly WalkthroughBeat[]
  position: number
  playing: boolean
  pace: WalkPace
  /** The sheet the walkthrough started on, then each call stepped into. */
  trail: readonly string[]
}>()

const emit = defineEmits<{ previous: []; next: []; toggle: []; restart: []; goTo: [index: number]; pace: [pace: WalkPace]; backTo: [depth: number] }>()

const PACES: readonly { readonly value: WalkPace; readonly label: string }[] = [
  { value: 'slow', label: '0.5×' },
  { value: 'normal', label: '1×' },
  { value: 'fast', label: '2×' }
]

const KEY = 'focus-ring grid size-8 shrink-0 place-items-center rounded-card text-text-muted transition-colors hover:bg-surface-raised hover:text-text disabled:pointer-events-none disabled:opacity-40'

function tickLabel(beat: WalkthroughBeat, index: number): string {
  return `Step ${index + 1}: ${beat.message.text}${beat.opens !== null ? ', has its own walkthrough' : ''}`
}
</script>

<template>
  <div class="glass pointer-events-auto flex w-full max-w-[760px] flex-col gap-1.5 rounded-card border border-border px-2 py-1.5 shadow-lift" role="group" aria-label="Walkthrough" data-testid="walkthrough-bar">
    <nav v-if="trail.length > 1" class="flex min-w-0 items-center gap-1 px-1 pt-0.5 text-caption" aria-label="Calls stepped into">
      <template v-for="(stop, depth) in trail" :key="depth">
        <svg v-if="depth > 0" viewBox="0 0 12 12" class="size-3 shrink-0 text-text-subtle" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" aria-hidden="true"><path d="m4.5 2.5 3.5 3.5-3.5 3.5" /></svg>
        <button
          v-if="depth < trail.length - 1"
          class="focus-ring max-w-[180px] truncate rounded-key px-1 font-mono text-text-muted hover:text-text"
          data-testid="walkthrough-trail"
          @click="emit('backTo', depth)"
        >
          {{ stop }}
        </button>
        <span v-else class="max-w-[220px] truncate font-mono font-semibold text-text" aria-current="step">{{ stop }}</span>
      </template>
    </nav>

    <div class="flex items-center gap-1">
      <button :class="KEY" title="Start over" aria-label="Start over" @click="emit('restart')">
        <svg viewBox="0 0 16 16" class="size-4" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M4 3.5v9M12 3.5 6.5 8l5.5 4.5Z" /></svg>
      </button>
      <button :class="KEY" title="Previous step (←)" aria-label="Previous step" data-testid="walkthrough-previous" @click="emit('previous')">
        <svg viewBox="0 0 16 16" class="size-4" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m10 3.5-4.5 4.5 4.5 4.5" /></svg>
      </button>
      <button
        class="key-gold focus-ring grid size-9 shrink-0 place-items-center rounded-full"
        :title="playing ? 'Pause (Space)' : 'Play (Space)'"
        :aria-label="playing ? 'Pause' : 'Play'"
        :aria-pressed="playing"
        data-testid="walkthrough-toggle"
        @click="emit('toggle')"
      >
        <svg v-if="playing" viewBox="0 0 16 16" class="size-4" fill="currentColor" aria-hidden="true"><rect x="4" y="3.5" width="2.6" height="9" rx="0.8" /><rect x="9.4" y="3.5" width="2.6" height="9" rx="0.8" /></svg>
        <svg v-else viewBox="0 0 16 16" class="size-4" fill="currentColor" aria-hidden="true"><path d="M5 3.3v9.4c0 .5.5.8.9.5l7-4.7c.4-.3.4-.8 0-1.1l-7-4.6c-.4-.3-.9 0-.9.5Z" /></svg>
      </button>
      <button :class="KEY" title="Next step (→)" aria-label="Next step" data-testid="walkthrough-next" @click="emit('next')">
        <svg viewBox="0 0 16 16" class="size-4" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m6 3.5 4.5 4.5L6 12.5" /></svg>
      </button>

      <span class="w-[4.5rem] shrink-0 text-center font-mono text-caption tabular-nums text-text-muted" data-testid="walkthrough-counter">{{ beats.length === 0 ? '–' : `${position + 1} / ${beats.length}` }}</span>

      <ol class="flex h-8 min-w-0 flex-1 items-center gap-[3px] overflow-hidden px-1" aria-label="Steps">
        <li v-for="(beat, index) in beats" :key="beat.key" class="flex h-full min-w-[3px] flex-1">
          <button
            class="tick focus-ring relative h-full w-full rounded-key"
            :class="[`motion-${beat.motion}`, index === position ? 'is-current' : index < position ? 'is-played' : 'is-ahead']"
            :style="{ '--depth': beat.depth }"
            :aria-label="tickLabel(beat, index)"
            :aria-current="index === position ? 'step' : undefined"
            :title="beat.message.text"
            @click="emit('goTo', index)"
          >
            <span v-if="beat.opens !== null" class="tick-opens" aria-hidden="true" />
          </button>
        </li>
      </ol>

      <div class="flex shrink-0 items-center rounded-control border border-border p-0.5" role="radiogroup" aria-label="Pace">
        <button
          v-for="choice in PACES"
          :key="choice.value"
          role="radio"
          :aria-checked="pace === choice.value"
          class="focus-ring h-6 rounded-key px-1.5 font-mono text-micro tabular-nums transition-colors"
          :class="pace === choice.value ? 'bg-surface-raised text-text' : 'text-text-subtle hover:text-text'"
          @click="emit('pace', choice.value)"
        >
          {{ choice.label }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* A tick's bar sits lower the deeper its call is nested, so the call tree reads as a skyline. */
.tick::before {
  content: '';
  position: absolute;
  inset-inline: 0;
  bottom: 6px;
  top: calc(6px + min(var(--depth), 4) * 3px);
  border-radius: 2px;
  background: var(--tick-tint);
  transition: background var(--motion-quick) ease;
}

.tick {
  --tick-tint: var(--color-border-strong);
}

.tick.motion-reply::before {
  top: calc(12px + min(var(--depth), 4) * 3px);
}

.tick.is-played {
  --tick-tint: color-mix(in srgb, var(--color-accent) 45%, var(--color-border-strong));
}

.tick.is-current {
  --tick-tint: var(--color-accent);
}

.tick.is-current::before {
  box-shadow: 0 0 8px -1px var(--color-accent);
}

.tick:hover {
  --tick-tint: var(--color-text-muted);
}

.tick-opens {
  position: absolute;
  left: 50%;
  top: 0;
  width: 4px;
  height: 4px;
  margin-left: -2px;
  border-radius: 999px;
  background: var(--color-accent);
}
</style>
