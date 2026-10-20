<script setup lang="ts">
import type { ViewMode } from '@/features/diagram/composables/useWalkthrough'

/**
 * Whether the open sequence is read as a static diagram or played as a walkthrough. Only a
 * sequence plays, so on any other sheet the walkthrough choice is unavailable.
 */
defineProps<{ mode: ViewMode; playable: boolean }>()

const emit = defineEmits<{ change: [mode: ViewMode] }>()

const CHOICES: readonly { readonly value: ViewMode; readonly label: string }[] = [
  { value: 'diagram', label: 'Diagram' },
  { value: 'walkthrough', label: 'Walkthrough' }
]
</script>

<template>
  <div class="glass flex h-10 items-center gap-0.5 rounded-card border border-border p-1 shadow-lift" role="radiogroup" aria-label="View" data-testid="view-mode">
    <button
      v-for="choice in CHOICES"
      :key="choice.value"
      role="radio"
      :aria-checked="mode === choice.value"
      :disabled="choice.value === 'walkthrough' && !playable"
      :title="choice.value === 'walkthrough' && !playable ? 'Only a sequence plays as a walkthrough' : undefined"
      class="focus-ring flex h-8 items-center gap-1.5 rounded-control px-2.5 text-small transition-colors disabled:cursor-not-allowed disabled:opacity-40"
      :class="mode === choice.value ? 'bg-surface-raised text-text shadow-[0_1px_2px_rgb(0_0_0/0.4)]' : 'text-text-subtle hover:text-text'"
      :data-testid="`view-mode-${choice.value}`"
      @click="emit('change', choice.value)"
    >
      <svg v-if="choice.value === 'walkthrough'" viewBox="0 0 16 16" class="size-3.5" fill="currentColor" aria-hidden="true"><path d="M5 3.3v9.4c0 .5.5.8.9.5l7-4.7c.4-.3.4-.8 0-1.1l-7-4.6c-.4-.3-.9 0-.9.5Z" /></svg>
      <svg v-else viewBox="0 0 16 16" class="size-3.5" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" aria-hidden="true"><path d="M3.5 2.5v11M12.5 2.5v11M3.5 6h9M12.5 10h-9" /></svg>
      {{ choice.label }}
    </button>
  </div>
</template>
