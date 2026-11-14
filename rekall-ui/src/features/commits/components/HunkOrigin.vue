<script setup lang="ts">
import type { ChangeSource } from '@/features/commits/lib/change-sources'

/** Who made a hunk: the commit and step, or the working tree. Pressing it opens just that change. */
defineProps<{ source: ChangeSource; stepNumber: number | null }>()

defineEmits<{ open: [] }>()
</script>

<template>
  <button
    type="button"
    class="focus-ring inline-flex h-5.5 min-w-0 max-w-1/2 cursor-pointer items-center gap-2 rounded-control border px-2 text-caption transition-colors duration-(--motion-quick)"
    :class="
      source.kind === 'working-tree'
        ? 'border-session/40 bg-session/15 text-session-strong hover:bg-session/25'
        : 'border-border-strong bg-surface-raised text-text-muted hover:bg-surface-hover hover:text-text'
    "
    :title="source.kind === 'working-tree' ? 'Read only the working tree' : `Read only this change: ${source.subject}`"
    data-testid="hunk-origin"
    @click="$emit('open')"
  >
    <span v-if="source.kind === 'working-tree'" class="size-1.5 shrink-0 rounded-full bg-session motion-safe:animate-pulse" aria-hidden="true" />
    <span class="shrink-0 font-mono text-micro">{{ source.commitHash?.slice(0, 7) ?? 'working tree' }}</span>
    <span v-if="stepNumber !== null" class="shrink-0 rounded-key border border-border-strong px-1 font-mono text-micro text-text-muted" data-testid="hunk-origin-step">{{ stepNumber }}</span>
    <span v-if="source.kind === 'commit'" class="min-w-0 truncate">{{ source.subject }}</span>
  </button>
</template>
