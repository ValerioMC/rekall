<script setup lang="ts">
import { computed } from 'vue'
import { compactTokens, estimatedTokens, SECTION_BODY_MAX_CHARACTERS } from '@/features/project/lib/section-size'

/** How full a section is, what reading it costs a session, and that no context carries it. */
const props = defineProps<{ characters: number }>()

const over = computed(() => props.characters > SECTION_BODY_MAX_CHARACTERS)
const fill = computed(() => `${Math.min(100, (props.characters / SECTION_BODY_MAX_CHARACTERS) * 100)}%`)
</script>

<template>
  <div class="flex flex-wrap items-center gap-x-4 gap-y-1 border-t border-border bg-canvas/40 px-5 py-2 text-caption text-text-muted" data-testid="section-gauge">
    <span class="whitespace-nowrap font-mono tabular-nums" :class="over ? 'text-danger' : 'text-text'">
      {{ characters.toLocaleString('en-US') }}
      <span class="text-text-subtle">of {{ SECTION_BODY_MAX_CHARACTERS.toLocaleString('en-US') }} characters</span>
    </span>
    <span class="h-1.5 w-40 shrink-0 overflow-hidden rounded-full bg-canvas shadow-[inset_0_1px_2px_rgb(0_0_0/0.6)]" aria-hidden="true">
      <span class="block h-full rounded-full" :class="over ? 'bg-danger' : 'bg-border-field'" :style="{ width: fill }" />
    </span>
    <span class="whitespace-nowrap font-mono tabular-nums">≈ {{ compactTokens(estimatedTokens(characters)) }} tokens when read</span>
    <span v-if="over" class="text-danger" role="alert" data-testid="section-gauge-over">Over the limit: this will not save.</span>
    <span v-else class="ml-auto flex items-center gap-1.5 whitespace-nowrap" data-testid="section-gauge-context">
      <svg class="size-3.5 text-text-subtle" viewBox="0 0 16 16" fill="none" aria-hidden="true">
        <path d="M2 8s2.2-4 6-4 6 4 6 4-2.2 4-6 4-6-4-6-4Z M3 13 13 3" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
        <circle cx="8" cy="8" r="1.7" stroke="currentColor" stroke-width="1.4" />
      </svg>
      Not in context. Costs one line, its title.
    </span>
  </div>
</template>
