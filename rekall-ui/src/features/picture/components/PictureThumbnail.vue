<script setup lang="ts">
import { ref } from 'vue'
import FileGlyph from '@/components/ui/FileGlyph.vue'
import { useSeen } from '@/composables/useSeen'

/** A picture asked for once its box has been in view. One that cannot be drawn keeps a plain file glyph. */
withDefaults(defineProps<{ src: string; alt?: string }>(), { alt: '' })

const box = ref<HTMLElement | null>(null)
const seen = useSeen(box)
const failed = ref(false)
</script>

<template>
  <span ref="box" class="relative grid place-items-center overflow-hidden rounded-key border border-border bg-canvas">
    <FileGlyph v-if="failed" />
    <img
      v-else-if="seen"
      :src="src"
      :alt="alt"
      class="absolute inset-0 size-full object-scale-down"
      draggable="false"
      data-testid="picture-thumbnail"
      @error="failed = true"
    />
  </span>
</template>
