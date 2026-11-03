<script setup lang="ts">
import { computed } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import { relativeTime } from '@/common/format/relative-time'
import type { Task } from '@/features/catalog/model/catalog'

/** "Quick" beside a quick task's title, and "Auto-accepted" with when, once Rekall closed it. */
const props = defineProps<{ task: Pick<Task, 'quick' | 'autoAccepted' | 'acceptedAt'>; compact?: boolean }>()

const autoAcceptedLabel = computed(() => {
  const when = props.task.acceptedAt && !props.compact ? relativeTime(props.task.acceptedAt) : ''
  return when ? `Auto-accepted ${when}` : 'Auto-accepted'
})
</script>

<template>
  <span v-if="task.quick" class="inline-flex shrink-0 items-center gap-1" data-testid="quick-task-mark">
    <AppBadge tone="neutral">Quick</AppBadge>
    <AppBadge
      v-if="task.autoAccepted"
      tone="safe"
      title="Rekall accepted it when the wrapup was written. Send it back to review it."
      data-testid="auto-accepted-mark"
    >
      {{ autoAcceptedLabel }}
    </AppBadge>
  </span>
</template>
