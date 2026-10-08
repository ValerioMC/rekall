<script setup lang="ts">
import TagIcon from '@/components/ui/TagIcon.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { TagId } from '@/model/branded'

/** The one tag a task carries, or none, chosen from the tags configured in the header. */
const tagId = defineModel<TagId | null>({ required: true })

const store = useConsoleStore()
</script>

<template>
  <p class="mb-1.5 mt-5 eyebrow text-caption">Tag</p>
  <div class="flex flex-wrap gap-1.5" data-testid="record-tag-picker">
    <button
      type="button"
      class="focus-ring flex h-8 items-center gap-1.5 rounded-key px-3 text-small"
      :class="tagId === null ? 'key-lit' : 'key-quiet'"
      :aria-pressed="tagId === null"
      @click="tagId = null"
    >
      No tag
    </button>
    <button
      v-for="option in store.tags"
      :key="option.id"
      type="button"
      class="focus-ring flex h-8 items-center gap-1.5 rounded-key px-3 text-small"
      :class="tagId === option.id ? 'key-lit' : 'key-quiet'"
      :aria-pressed="tagId === option.id"
      @click="tagId = option.id"
    >
      <TagIcon :icon="option.icon" :color="option.color" :size="14" />
      {{ option.name }}
    </button>
  </div>
  <p v-if="store.tags.length === 0" class="mt-1.5 text-caption text-text-subtle">
    No tags configured yet. Add one from the tags button in the header.
  </p>
</template>
