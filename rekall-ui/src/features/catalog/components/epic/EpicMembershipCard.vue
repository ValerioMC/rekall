<script setup lang="ts">
import EpicGlyph from '@/features/catalog/components/epic/EpicGlyph.vue'
import EpicPageLink from '@/features/catalog/components/epic/EpicPageLink.vue'
import type { Task } from '@/features/catalog/model/catalog'

/** Above a task's dossier: the epic it belongs to, whose description and notes reach its context too. */
defineProps<{ epic: Task }>()
</script>

<template>
  <div
    class="mb-2 flex items-center gap-2.5 rounded-control border border-epic-line bg-epic-soft py-1.5 pl-2.5 pr-1"
    data-testid="epic-membership"
  >
    <EpicGlyph :size="14" />
    <span class="min-w-0 flex-1">
      <span class="block truncate text-small font-medium text-text">{{ epic.title }}</span>
      <span class="block truncate text-micro text-text-subtle">Its description and notes travel with this task</span>
    </span>
    <EpicPageLink :epic-id="epic.id" :epic-title="epic.title" />
  </div>
</template>
