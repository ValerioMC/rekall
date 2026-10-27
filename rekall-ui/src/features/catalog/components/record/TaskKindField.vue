<script setup lang="ts">
import EpicGlyph from '@/features/catalog/components/epic/EpicGlyph.vue'
import type { TaskKind } from '@/features/catalog/model/catalog'

/** Whether the record is a task with its own work, or an epic that gathers tasks. */
const kind = defineModel<TaskKind>({ required: true })
</script>

<template>
  <p class="mb-1.5 mt-5 eyebrow text-caption">Kind</p>
  <div class="flex flex-wrap gap-1.5" data-testid="record-kind-picker">
    <button
      type="button"
      class="focus-ring flex h-8 items-center gap-1.5 rounded-key px-3 text-small"
      :class="kind === 'TASK' ? 'key-lit' : 'key-quiet'"
      :aria-pressed="kind === 'TASK'"
      @click="kind = 'TASK'"
    >
      Task
    </button>
    <button
      type="button"
      class="focus-ring flex h-8 items-center gap-1.5 rounded-key px-3 text-small"
      :class="kind === 'EPIC' ? 'key-lit' : 'key-quiet'"
      :aria-pressed="kind === 'EPIC'"
      data-testid="record-kind-epic"
      @click="kind = 'EPIC'"
    >
      <EpicGlyph :size="14" />
      Epic
    </button>
  </div>
  <p v-if="kind === 'EPIC'" class="mt-1.5 text-caption text-text-subtle">
    An epic holds no steps. Tasks of its project belong to it and inherit its description and notes.
  </p>
</template>
