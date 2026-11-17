<script setup lang="ts">
import { computed } from 'vue'
import { compactTokens } from '@/features/project/lib/section-size'
import { contextShareOf, type ContextPartKey, type ContextShareInput } from '@/features/project/lib/context-share'

/** What a session is handed when it loads the project, and what stays out until it asks. */
const props = defineProps<{ input: ContextShareInput }>()

const PART_COLOUR: Readonly<Record<ContextPartKey, string>> = {
  blueprint: 'bg-accent',
  components: 'bg-anchor',
  connections: 'bg-filed',
  pages: 'bg-session'
}

const share = computed(() => contextShareOf(props.input))
</script>

<template>
  <aside
    class="flex min-h-0 flex-col overflow-y-auto rounded-card border border-border bg-surface shadow-[var(--shadow-raised)]"
    aria-label="What a session is handed"
    data-testid="context-panel"
  >
    <h2 class="border-b border-border px-5 py-3 text-title font-semibold tracking-[-0.01em] text-text">Handed to a session</h2>
    <div class="flex flex-col gap-4 p-5">
      <div class="flex items-baseline justify-between gap-3">
        <span class="font-mono text-heading font-medium tabular-nums text-text" data-testid="context-total">≈ {{ compactTokens(share.totalTokens) }}</span>
        <span class="text-caption text-text-subtle">tokens, in every context of this project</span>
      </div>
      <div class="flex h-2 gap-0.5" aria-hidden="true">
        <span
          v-for="part in share.parts.filter((candidate) => candidate.tokens > 0)"
          :key="part.key"
          class="h-full rounded-key"
          :class="PART_COLOUR[part.key]"
          :style="{ flex: part.tokens }"
        />
      </div>
      <ul class="flex flex-col">
        <li
          v-for="part in share.parts"
          :key="part.key"
          class="grid grid-cols-[10px_minmax(0,1fr)_auto] items-baseline gap-x-2.5 border-t border-border py-2.5 first:border-t-0"
          :data-testid="`context-part-${part.key}`"
        >
          <span class="size-2 self-center rounded-key" :class="PART_COLOUR[part.key]" aria-hidden="true" />
          <span class="text-small font-medium text-text">{{ part.label }}</span>
          <span class="font-mono text-caption tabular-nums text-text-muted">{{ part.tokens.toLocaleString('en-US') }}</span>
          <span class="col-span-2 col-start-2 mt-0.5 text-caption text-text-subtle">{{ part.detail }}</span>
        </li>
      </ul>
      <p class="border-t border-border-strong pt-3.5 text-caption leading-relaxed text-text-subtle" data-testid="context-held-back">
        Kept out of context: the text of every page,
        <span class="font-mono tabular-nums text-text-muted">{{ share.heldBackCharacters.toLocaleString('en-US') }}</span>
        characters. A session reads one with <code class="font-mono text-anchor">rekall_read_sections</code> when the work touches it.
      </p>
    </div>
  </aside>
</template>
