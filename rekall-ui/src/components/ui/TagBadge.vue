<script setup lang="ts">
import { computed } from 'vue'
import TagIcon from '@/components/ui/TagIcon.vue'
import { tagColor } from '@/features/tags/model/tag'

const props = withDefaults(
  defineProps<{
    name: string
    icon: string
    color: string
    size?: 'sm' | 'md'
  }>(),
  { size: 'sm' }
)

const style = computed(() => ({ '--tag-tint': tagColor(props.color).base }))
</script>

<template>
  <span
    class="tag-chip inline-flex min-w-0 items-center gap-1 rounded-key font-medium"
    :class="size === 'sm' ? 'h-[19px] px-1.5 text-micro' : 'h-6 px-2 text-caption'"
    :style="style"
    :title="name"
  >
    <TagIcon :icon="icon" :color="color" :size="size === 'sm' ? 11 : 13" />
    <span class="truncate">{{ name }}</span>
  </span>
</template>
