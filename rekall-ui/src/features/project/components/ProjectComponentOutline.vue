<script setup lang="ts">
import type { OutlineHeading } from '@/features/project/lib/markdown-outline'

/** The headings of the description beside it, each a jump to its place in the page. */
defineProps<{ headings: readonly OutlineHeading[] }>()

const emit = defineEmits<{ jump: [text: string] }>()
</script>

<template>
  <nav class="sticky top-0 flex flex-col gap-0.5" aria-label="On this page" data-testid="component-outline">
    <p class="mb-1 text-caption text-text-subtle">On this page</p>
    <button
      v-for="heading in headings"
      :key="`${heading.level}:${heading.text}`"
      type="button"
      class="focus-ring cursor-pointer truncate rounded-key py-1 text-left transition-colors hover:text-text"
      :class="heading.level === 2 ? 'text-small text-text-muted' : 'pl-3 text-caption text-text-subtle'"
      @click="emit('jump', heading.text)"
    >
      {{ heading.text }}
    </button>
  </nav>
</template>
