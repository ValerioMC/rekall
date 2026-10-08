<script setup lang="ts">
/** A task with no description yet: what the brief is for, how it differs from the other three surfaces, and the key that starts it. */
import AppButton from '@/components/ui/AppButton.vue'

defineEmits<{ write: [] }>()
</script>

<template>
  <div class="min-h-0 flex-1 overflow-y-auto">
    <div class="max-w-[620px] px-9 py-12">
      <p class="eyebrow">
        Description
      </p>
      <h2 class="mb-1.5 mt-1.5 text-display font-semibold leading-tight tracking-[-0.025em] text-text">
        Nothing says what this task is
      </h2>
      <p class="mb-6 text-body leading-relaxed text-text-muted">
        The brief: what has to be built, what it has to satisfy, what is out of scope. Markdown,
        at whatever length the work needs, corrected when the work is redefined rather than
        when it moves.
      </p>

      <AppButton
        variant="primary"
        data-testid="write-description"
        @click="$emit('write')"
      >
        Write the description
      </AppButton>

      <ul class="mt-9 border-t border-border">
        <li
          v-for="surface in [
            {
              name: 'Description',
              asks: 'What is this task?',
              says: 'Changes when the work is redefined.',
              glyph: 'page'
            },
            {
              name: 'Steps',
              asks: 'What is left of it?',
              says: 'Ticked by hand as the work is reviewed.',
              glyph: 'check'
            },
            {
              name: 'Wrapup',
              asks: 'Where did the implementation get to?',
              says: 'Rewritten in place, never appended to.',
              glyph: 'diamond'
            },
            {
              name: 'Note',
              asks: 'What did I learn?',
              says: 'Keeps its title, and can be on several tasks.',
              glyph: 'lines'
            }
          ]"
          :key="surface.name"
          class="flex items-start gap-3 border-b border-border py-3"
        >
          <svg
            class="mt-0.5 size-3 shrink-0"
            :class="surface.name === 'Description' ? 'text-accent' : 'text-text-subtle'"
            viewBox="0 0 12 12"
            fill="none"
            aria-hidden="true"
          >
            <template v-if="surface.glyph === 'page'">
              <path d="M2.6 1.1h4.1l2.7 2.7v7.1H2.6z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" />
              <path d="M4.5 6.1h3.2M4.5 8.2h2" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" />
            </template>
            <template v-else-if="surface.glyph === 'check'">
              <path d="M1.2 3.4 2.6 4.8l2.4-2.6" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
              <path d="M6.8 3.6h4M6.8 8.4h4M1.4 8.4h3.4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
            </template>
            <path
              v-else-if="surface.glyph === 'diamond'"
              d="M6 1.2 10.8 6 6 10.8 1.2 6z"
              stroke="currentColor"
              stroke-width="1.3"
              stroke-linejoin="round"
            />
            <path
              v-else
              d="M1.6 3h8.8M1.6 6h8.8M1.6 9h5.4"
              stroke="currentColor"
              stroke-width="1.2"
              stroke-linecap="round"
            />
          </svg>
          <span class="min-w-0 flex-1">
            <span class="flex flex-wrap items-baseline gap-x-2.5">
              <span class="text-small font-semibold text-text">{{ surface.name }}</span>
              <span class="text-small text-text-muted">{{ surface.asks }}</span>
            </span>
            <span class="mt-0.5 block text-caption text-text-subtle">{{ surface.says }}</span>
          </span>
        </li>
      </ul>
    </div>
  </div>
</template>
