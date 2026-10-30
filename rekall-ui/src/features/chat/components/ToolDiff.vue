<script setup lang="ts">
import { computed, ref } from 'vue'
import type { DiffLine } from '@/features/chat/lib/tool-diff'

/**
 * The lines an edit takes out and puts in. The sign in the gutter carries the meaning, the
 * background only echoes it. Past `limit` lines the rest waits behind a key.
 */
const props = withDefaults(defineProps<{ lines: readonly DiffLine[]; limit?: number }>(), { limit: 40 })

const expanded = ref(false)

const hidden = computed(() => Math.max(0, props.lines.length - props.limit))
const shown = computed(() => (expanded.value ? props.lines : props.lines.slice(0, props.limit)))
</script>

<template>
  <div class="overflow-hidden rounded-control border border-border font-mono text-small" data-testid="tool-diff">
    <div
      v-for="(line, index) in shown"
      :key="index"
      class="flex gap-2 px-2 whitespace-pre-wrap"
      :class="line.sign === '+' ? 'bg-safe-soft text-text' : 'bg-danger-soft text-text'"
      :data-sign="line.sign"
    >
      <span class="w-3 shrink-0 select-none text-center" :class="line.sign === '+' ? 'text-safe' : 'text-danger'" aria-hidden="true">{{
        line.sign === '+' ? '+' : '−'
      }}</span>
      <span class="sr-only">{{ line.sign === '+' ? 'added' : 'removed' }}</span>
      <span class="min-w-0 break-words">{{ line.text }}</span>
    </div>
    <button
      v-if="hidden > 0 && !expanded"
      type="button"
      class="focus-ring w-full cursor-pointer bg-surface px-2 py-1 text-left text-caption text-text-muted hover:text-text"
      @click="expanded = true"
    >
      Show {{ hidden }} more {{ hidden === 1 ? 'line' : 'lines' }}
    </button>
  </div>
</template>
