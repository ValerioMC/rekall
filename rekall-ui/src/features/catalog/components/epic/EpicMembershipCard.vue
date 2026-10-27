<script setup lang="ts">
import EpicGlyph from '@/features/catalog/components/epic/EpicGlyph.vue'
import type { Task } from '@/features/catalog/model/catalog'

/** Above a task's dossier: the epic it belongs to, whose description and notes reach its context too. A click opens the epic. */
defineProps<{ epic: Task }>()

defineEmits<{ open: [] }>()
</script>

<template>
  <button
    type="button"
    class="focus-ring mb-2 flex w-full items-center gap-2.5 rounded-control border border-epic-line bg-epic-soft py-1.5 pl-2.5 pr-2 text-left transition-colors hover:border-epic"
    :title="`Open the epic ${epic.title}`"
    data-testid="epic-membership"
    @click="$emit('open')"
  >
    <EpicGlyph :size="14" />
    <span class="min-w-0 flex-1">
      <span class="block truncate text-small font-medium text-text">{{ epic.title }}</span>
      <span class="block truncate text-micro text-text-subtle">Its description and notes travel with this task</span>
    </span>
    <svg class="size-3 shrink-0 text-text-subtle" viewBox="0 0 12 12" fill="none" aria-hidden="true">
      <path d="M4.5 2.5 8 6l-3.5 3.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
  </button>
</template>
