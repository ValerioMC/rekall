<script setup lang="ts">
import EpicGlyph from '@/features/catalog/components/epic/EpicGlyph.vue'

/** On a task's row: the epic it belongs to, by title, cut short before it crowds the anchor. */
defineProps<{ title: string }>()
</script>

<template>
  <span
    class="inline-flex min-w-0 max-w-[55%] shrink items-center gap-1 rounded-full border border-epic-line bg-epic-soft py-px pl-1 pr-1.5 text-micro leading-[15px] text-epic"
    :title="`Part of the epic ${title}`"
    data-testid="task-epic"
  >
    <EpicGlyph :size="10" />
    <span class="min-w-0 truncate">{{ title }}</span>
  </span>
</template>
