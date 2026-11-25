<script setup lang="ts">
import { computed } from 'vue'
import { COMPOSITION_SEGMENTS } from '@/features/catalog/lib/composition-segments'
import type { ProjectComposition } from '@/features/catalog/lib/project-composition'

const props = defineProps<{ composition: ProjectComposition }>()

const shown = computed(() => COMPOSITION_SEGMENTS.filter((segment) => props.composition[segment.key] > 0))
const summary = computed(
  () => shown.value.map((segment) => `${props.composition[segment.key]} ${segment.label}`).join(', ') || 'No tasks'
)
</script>

<template>
  <div class="trough flex h-[6px] w-full gap-[2px] overflow-hidden p-[1px]" role="img" :aria-label="summary" data-testid="project-composition">
    <span
      v-for="segment in shown"
      :key="segment.key"
      class="tube-gold h-full min-w-[3px] rounded-full"
      :style="{ flexGrow: composition[segment.key], flexBasis: '0%', '--tube': segment.tube }"
    />
  </div>
</template>
