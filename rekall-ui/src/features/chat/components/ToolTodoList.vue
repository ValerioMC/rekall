<script setup lang="ts">
import { computed } from 'vue'
import TodoStatusGlyph from './TodoStatusGlyph.vue'
import { inProgressFirst } from '@/features/chat/lib/todo-list'
import type { TodoItem } from '@/features/chat/model/chat-event'

const props = defineProps<{ items: readonly TodoItem[] }>()

const ordered = computed(() => inProgressFirst(props.items))
</script>

<template>
  <ul class="flex flex-col gap-1" data-testid="tool-todos">
    <li v-for="(item, index) in ordered" :key="index" class="flex items-start gap-2 text-small">
      <TodoStatusGlyph class="mt-0.5" :status="item.status" />
      <span
        class="min-w-0 break-words"
        :class="item.status === 'completed' ? 'text-text-subtle line-through' : item.status === 'inProgress' ? 'text-text' : 'text-text-muted'"
        >{{ item.content }}</span
      >
    </li>
  </ul>
</template>
