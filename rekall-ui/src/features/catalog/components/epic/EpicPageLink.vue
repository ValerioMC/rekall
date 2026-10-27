<script setup lang="ts">
import { computed } from 'vue'
import type { TaskId } from '@/model/branded'

/** One click from wherever an epic is shown to its dashboard. Always visible, never a hover target. */
const props = defineProps<{ epicId: TaskId; epicTitle: string }>()

const label = computed(() => `Open the ${props.epicTitle} dashboard`)
</script>

<template>
  <RouterLink
    data-testid="open-epic-page"
    class="focus-ring grid size-7 shrink-0 place-items-center rounded-control text-text-subtle transition-colors duration-(--motion-quick) hover:bg-epic-soft hover:text-epic focus-visible:bg-epic-soft focus-visible:text-epic"
    :to="{ name: 'epic', params: { id: epicId } }"
    :aria-label="label"
    :title="label"
  >
    <svg class="size-3.5" viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <path d="M4 4h7v7H4zM13 4h7v4h-7zM13 10h7v10h-7zM4 13h7v7H4z" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round" />
    </svg>
  </RouterLink>
</template>
