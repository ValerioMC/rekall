<script setup lang="ts">
import { relativeTime } from '@/common/format/relative-time'
import type { CommitReference } from '@/model/commitReference'

withDefaults(
  defineProps<{
    references: readonly CommitReference[]
    /** Show which step each row belongs to. Off inside a step's own detail, where it's implied. */
    showStepTag?: boolean
    dense?: boolean
  }>(),
  { showStepTag: false, dense: false }
)
</script>

<template>
  <ul
    v-if="references.length"
    class="min-w-0 space-y-1"
    :class="dense ? 'mt-2' : 'mt-1'"
    data-testid="commit-reference-list"
  >
    <li
      v-for="reference in references"
      :key="reference.id"
      class="group/commit flex min-w-0 items-center gap-2 rounded-[var(--radius-control)] border border-transparent px-1.5 py-1 transition-colors hover:border-border hover:bg-surface-raised"
      data-testid="commit-reference-row"
    >
      <svg
        class="size-3 shrink-0 text-text-subtle transition-colors group-hover/commit:text-accent"
        viewBox="0 0 12 12"
        fill="none"
        aria-hidden="true"
      >
        <path d="M1 6h2.7M8.3 6H11" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
        <circle cx="6" cy="6" r="2.15" stroke="currentColor" stroke-width="1.2" />
      </svg>

      <span
        class="shrink-0 rounded-[6px] border border-border-strong bg-surface px-1.5 py-0.5 font-mono text-[10.5px] text-accent"
        data-testid="commit-reference-hash"
      >
        {{ reference.commitHash.slice(0, 7) }}
      </span>

      <span class="min-w-0 flex-1 truncate text-[12px] text-text-muted" :title="reference.comment">
        {{ reference.comment }}
      </span>

      <span
        v-if="showStepTag && reference.stepTitle"
        class="shrink-0 truncate rounded-full border border-border-strong px-1.5 py-px text-[10px] text-text-subtle"
        data-testid="commit-reference-step-tag"
      >
        {{ reference.stepTitle }}
      </span>
      <span
        v-else-if="showStepTag"
        class="shrink-0 rounded-full border border-dashed border-border-strong px-1.5 py-px text-[10px] text-text-subtle"
      >
        task
      </span>

      <span class="shrink-0 text-[10.5px] text-text-subtle">{{ relativeTime(reference.createdAt) }}</span>
    </li>
  </ul>
</template>
