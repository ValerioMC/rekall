<script setup lang="ts">
import { computed } from 'vue'
import { TASK_STATUS_LABEL, TASK_STATUS_ORDER } from '@/model/catalog'
import type { Task, TaskStatus } from '@/model/catalog'

const props = withDefaults(defineProps<{ tasks: readonly Task[]; size?: 'sm' | 'md' }>(), {
  size: 'sm'
})

const TUBE_COLOR: Readonly<Record<TaskStatus, string>> = {
  IN_PROGRESS: 'var(--color-accent)',
  TODO: 'var(--color-text-subtle)',
  BLOCKED: 'var(--color-danger)',
  BACKLOG: 'var(--color-border-strong)',
  DONE: 'var(--color-safe)'
}

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
    class="trough flex w-full gap-[2px] overflow-hidden p-[1px]"
    :class="size === 'sm' ? 'h-[6px]' : 'h-2'"
    role="img"
    :aria-label="summary"
  >
    <span
      v-for="segment in segments"
      :key="segment.status"
      class="tube-gold h-full min-w-[3px] rounded-full"
      :style="{ flexGrow: segment.count, flexBasis: '0%', '--tube': TUBE_COLOR[segment.status] }"
    />
  </div>
</template>
