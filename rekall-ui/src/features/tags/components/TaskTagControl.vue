<script setup lang="ts">
import { ref } from 'vue'
import { storeToRefs } from 'pinia'
import TagPicker from '@/features/tags/components/TagPicker.vue'
import TagBadge from '@/components/ui/TagBadge.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useConsoleStore } from '@/stores/console.store'
import type { TagId } from '@/model/branded'
import type { Task } from '@/features/catalog/model/catalog'

/**
 * The task's tag as a badge that opens the tag list, or, with none, a dashed "New tag" button
 * shaped like the skills' "Add a skill". A pick is saved at once.
 */
const props = defineProps<{ task: Task }>()

const store = useConsoleStore()
const { tags } = storeToRefs(store)
const { run } = useAsyncAction()

const open = ref(false)
const trigger = ref<HTMLElement | null>(null)

function pick(tagId: TagId | null): void {
  open.value = false
  void run(() => store.setTaskTag(props.task.id, tagId))
}
</script>

<template>
  <span class="inline-flex min-w-0" data-testid="task-tag-control">
    <button
      ref="trigger"
      type="button"
      class="focus-ring inline-flex h-6 min-w-0 cursor-pointer items-center transition-colors duration-(--motion-quick)"
      :class="
        task.tagId
          ? 'rounded-key hover:brightness-125'
          : 'gap-1 rounded-key border border-dashed border-border-strong px-2 text-caption text-text-muted hover:border-anchor hover:text-anchor'
      "
      aria-haspopup="dialog"
      :aria-expanded="open"
      :aria-label="task.tagId ? `Tag: ${task.tagName}. Change it` : 'Add a tag to this task'"
      data-testid="task-tag-trigger"
      @click="open = !open"
    >
      <TagBadge
        v-if="task.tagId && task.tagName && task.tagIcon && task.tagColor"
        :name="task.tagName"
        :icon="task.tagIcon"
        :color="task.tagColor"
        size="md"
      />
      <template v-else>
        <svg class="size-3 shrink-0" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <path
            d="M1.5 6.2V2.5a1 1 0 0 1 1-1h3.7l4.3 4.3a1 1 0 0 1 0 1.4L7.2 10a1 1 0 0 1-1.4 0L1.5 6.2Z"
            stroke="currentColor"
            stroke-width="1.1"
            stroke-linejoin="round"
          />
          <circle cx="4.3" cy="4.3" r=".8" fill="currentColor" />
        </svg>
        New tag
      </template>
    </button>
    <TagPicker
      v-if="open && trigger"
      :tags="tags"
      :selected="task.tagId"
      :anchor="trigger"
      @select="pick"
      @close="open = false"
    />
  </span>
</template>
