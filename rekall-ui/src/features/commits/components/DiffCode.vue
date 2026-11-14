<script setup lang="ts">
import type { CodePart, CodeTone } from '@/features/commits/lib/code-tokens'

/** One line of code as drawn pieces; `side` colours the words an edit changed. */
defineProps<{ parts: readonly CodePart[]; side: 'added' | 'removed' | null }>()

const TONE_CLASS: Readonly<Record<CodeTone, string>> = {
  plain: '',
  keyword: 'font-semibold text-text',
  string: 'text-text-muted',
  comment: 'italic text-text-subtle'
}
</script>

<template>
  <template v-for="(part, index) in parts" :key="index">
    <mark
      v-if="part.changed"
      class="rounded-key px-px text-inherit"
      :class="[TONE_CLASS[part.tone], side === 'added' ? 'bg-safe/35' : 'bg-danger/35']"
      data-testid="diff-word"
      >{{ part.text }}</mark
    >
    <span v-else :class="TONE_CLASS[part.tone]">{{ part.text }}</span>
  </template>
</template>
