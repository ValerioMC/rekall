<script setup lang="ts">
import { computed } from 'vue'
import TagIcon from '@/components/ui/TagIcon.vue'
import { tagColor } from '@/model/tag'

const props = withDefaults(
  defineProps<{
    name: string
    icon: string
    color: string
    size?: 'sm' | 'md'
  }>(),
  { size: 'sm' }
)

const style = computed(() => {
  const palette = tagColor(props.color)
  return { '--tag-tint': palette.base, '--tag-line': palette.line, color: palette.base }
})
</script>

<template>
  <span
    class="tag-chip inline-flex min-w-0 items-center gap-1 rounded-full font-medium"
    :class="size === 'sm' ? 'h-[19px] px-1.5 text-[10px]' : 'h-6 px-2 text-[11.5px]'"
    :style="style"
    :title="name"
  >
    <TagIcon :icon="icon" :color="color" :size="size === 'sm' ? 11 : 13" :glow="false" />
    <span class="truncate">{{ name }}</span>
  </span>
</template>
