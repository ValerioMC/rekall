<script setup lang="ts">
import { computed } from 'vue'
import { TASK_STATUS_COLOR, TASK_STATUS_LABEL, TASK_STATUS_ORDER } from '@/model/catalog'
import type { Task } from '@/model/catalog'

const props = withDefaults(defineProps<{ tasks: readonly Task[]; size?: 'sm' | 'md' }>(), {
  size: 'sm'
})

const segments = computed(() =>
  TASK_STATUS_ORDER.map((status) => ({
    status,
    count: props.tasks.filter((t) => t.status === status).length
  })).filter((s) => s.count > 0)
)

const summary = computed(
  () => segments.value.map((s) => `${s.count} ${TASK_STATUS_LABEL[s.status]}`).join(', ') || 'No tasks'
)
</script>

<template>
  <!-- Segments are separated by a hairline gap rather than butted together: two statuses in
       adjacent hues (in progress and blocked) otherwise blur into one colour at 4px tall. -->
  <div
    class="flex w-full gap-[2px] overflow-hidden rounded-full"
    :class="[size === 'sm' ? 'h-1' : 'h-1.5', segments.length ? '' : 'bg-border']"
    role="img"
    :aria-label="summary"
  >
    <span
      v-for="segment in segments"
      :key="segment.status"
      class="h-full min-w-[3px] rounded-full"
      :class="TASK_STATUS_COLOR[segment.status]"
      :style="{ flexGrow: segment.count, flexBasis: '0%' }"
    />
  </div>
</template>
