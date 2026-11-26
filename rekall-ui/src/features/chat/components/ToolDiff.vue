<script setup lang="ts">
import { computed, ref } from 'vue'
import type { DiffLine } from '@/features/chat/lib/tool-diff'

/**
 * The lines of one file's change. The sign in the gutter carries the meaning, the background only
 * echoes it. Past `limit` lines the rest waits behind a key.
 */
const props = withDefaults(defineProps<{ lines: readonly DiffLine[]; limit?: number }>(), { limit: 40 })

interface LineLook {
  readonly row: string
  readonly gutter: string
  readonly mark: string
  readonly spoken: string | null
}

const LOOKS: Readonly<Record<DiffLine['sign'], LineLook>> = {
  '+': { row: 'bg-safe-soft text-text', gutter: 'text-safe', mark: '+', spoken: 'added' },
  '-': { row: 'bg-danger-soft text-text', gutter: 'text-danger', mark: '−', spoken: 'removed' },
  ' ': { row: 'text-text-muted', gutter: '', mark: '', spoken: null },
  '@': { row: 'bg-surface-raised py-0.5 text-caption text-text-subtle', gutter: '', mark: '', spoken: null }
}

const expanded = ref(false)

const hidden = computed(() => Math.max(0, props.lines.length - props.limit))
const shown = computed(() => (expanded.value ? props.lines : props.lines.slice(0, props.limit)))
</script>

<template>
  <div class="font-mono text-small" data-testid="tool-diff">
    <div
      v-for="(line, index) in shown"
      :key="index"
      class="flex gap-2 px-3 whitespace-pre-wrap"
      :class="LOOKS[line.sign].row"
      :data-sign="line.sign"
    >
      <span class="w-3 shrink-0 select-none text-center" :class="LOOKS[line.sign].gutter" aria-hidden="true">{{
        LOOKS[line.sign].mark
      }}</span>
      <span v-if="LOOKS[line.sign].spoken" class="sr-only">{{ LOOKS[line.sign].spoken }}</span>
      <span class="min-w-0 break-words">{{ line.text }}</span>
    </div>
    <button
      v-if="hidden > 0 && !expanded"
      type="button"
      class="focus-ring w-full cursor-pointer border-t border-border px-3 py-1 text-left font-sans text-caption text-text-muted hover:bg-surface-raised hover:text-text"
      @click="expanded = true"
    >
      Show {{ hidden }} more {{ hidden === 1 ? 'line' : 'lines' }}
    </button>
  </div>
</template>
