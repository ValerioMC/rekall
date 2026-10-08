<script setup lang="ts">
import { splitByRanges, type PathRow } from '@/common/path/path-query'

/** One folder or file of the folder in view, with the letters the filter matched lit. */
defineProps<{
  id: string
  row: PathRow
  highlighted: boolean
}>()

defineEmits<{ hover: []; activate: []; insert: [] }>()
</script>

<template>
  <li
    :id="id"
    role="option"
    :aria-selected="highlighted"
    class="path-row group/row flex h-[30px] cursor-pointer items-center gap-2.5 rounded-control pl-3.5 pr-1.5"
    :class="[highlighted ? 'selected-row' : 'hover:bg-surface-hover', row.entry.hidden && 'opacity-60']"
    :data-directory="row.entry.directory"
    data-testid="path-picker-row"
    @mousemove="$emit('hover')"
    @click="$emit('activate')"
  >
    <svg
      v-if="row.entry.directory"
      class="size-3.5 shrink-0"
      :class="highlighted ? 'text-accent' : 'text-text-muted'"
      viewBox="0 0 16 16"
      aria-hidden="true"
    >
      <path
        d="M1.75 4.25c0-.55.45-1 1-1h3.1l1.4 1.5h6c.55 0 1 .45 1 1v6.5c0 .55-.45 1-1 1H2.75c-.55 0-1-.45-1-1v-8Z"
        fill="currentColor"
        fill-opacity="0.16"
        stroke="currentColor"
        stroke-width="1.25"
        stroke-linejoin="round"
      />
    </svg>
    <svg v-else class="size-3.5 shrink-0 text-text-subtle" viewBox="0 0 16 16" fill="none" aria-hidden="true">
      <path
        d="M4 1.75h5.1L12.25 5v8.25c0 .55-.45 1-1 1H4c-.55 0-1-.45-1-1V2.75c0-.55.45-1 1-1Z"
        stroke="currentColor"
        stroke-width="1.25"
        stroke-linejoin="round"
      />
      <path d="M9 1.9V5.1h3.1" stroke="currentColor" stroke-width="1.25" stroke-linejoin="round" />
    </svg>

    <span
      class="min-w-0 flex-1 truncate font-mono text-small"
      :class="highlighted ? 'text-text' : 'text-text-muted'"
      :title="row.entry.name"
    >
      <template v-for="(part, partIndex) in splitByRanges(row.entry.name, row.ranges)" :key="partIndex">
        <mark v-if="part.matched" class="path-match">{{ part.text }}</mark>
        <template v-else>{{ part.text }}</template>
      </template>
    </span>

    <!-- On the highlighted folder, both of its moves, since a click only does one. -->
    <span v-if="row.entry.directory && highlighted" class="flex shrink-0 items-center gap-1 text-micro text-text-subtle">
      <button
        type="button"
        tabindex="-1"
        class="rounded-control px-1.5 py-0.5 transition-colors hover:bg-accent-soft hover:text-accent"
        data-testid="path-picker-insert-folder"
        @click.stop="$emit('insert')"
      >
        <kbd>↵</kbd> insert
      </button>
    </span>
    <svg
      v-if="row.entry.directory"
      class="size-3 shrink-0 transition-colors"
      :class="highlighted ? 'text-text-muted' : 'text-text-subtle'"
      viewBox="0 0 12 12"
      fill="none"
      aria-hidden="true"
    >
      <path d="m4.5 2.75 3.25 3.25-3.25 3.25" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
  </li>
</template>

<style scoped>
/* The matched letters: lit, not boxed, so a scattered match still reads as one word. */
.path-match {
  background: none;
  color: var(--color-accent-strong);
  font-weight: 600;
}
</style>
