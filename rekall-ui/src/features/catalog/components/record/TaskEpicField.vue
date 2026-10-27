<script setup lang="ts">
import EpicGlyph from '@/features/catalog/components/epic/EpicGlyph.vue'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskId } from '@/model/branded'

/** The epic a task belongs to, or none, chosen from the epics of its project. */
const epicId = defineModel<TaskId | null>({ required: true })

defineProps<{ epics: readonly Task[] }>()
</script>

<template>
  <p class="mb-1.5 mt-5 eyebrow text-caption">Epic</p>
  <div class="flex flex-wrap gap-1.5" data-testid="record-epic-picker">
    <button
      type="button"
      class="focus-ring flex h-8 items-center gap-1.5 rounded-key px-3 text-small"
      :class="epicId === null ? 'key-lit' : 'key-quiet'"
      :aria-pressed="epicId === null"
      @click="epicId = null"
    >
      No epic
    </button>
    <button
      v-for="epic in epics"
      :key="epic.id"
      type="button"
      class="focus-ring flex h-8 max-w-full items-center gap-1.5 rounded-key px-3 text-small"
      :class="epicId === epic.id ? 'key-lit' : 'key-quiet'"
      :aria-pressed="epicId === epic.id"
      @click="epicId = epic.id"
    >
      <EpicGlyph :size="14" />
      <span class="truncate">{{ epic.title }}</span>
    </button>
  </div>
  <p v-if="epics.length === 0" class="mt-1.5 text-caption text-text-subtle">
    No epic in this project yet. Create a task of kind Epic to gather tasks under it.
  </p>
</template>
