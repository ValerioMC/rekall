<script setup lang="ts">
import { computed } from 'vue'
import CopyAnchorButton from '@/components/console/CopyAnchorButton.vue'
import TaskStatusMenu from '@/features/work/components/TaskStatusMenu.vue'
import type { WorkRow } from '@/features/work/model/work-view'
import { rkCommand } from '@/common/format/rk-command'

/** The top of the properties bar: the status pill that changes the status, and the copyable `/rk` anchor. */
const props = defineProps<{ row: WorkRow }>()

const task = computed(() => props.row.task)
</script>

<template>
  <div class="flex min-w-0 items-center gap-2 border-b border-border px-4 py-3" data-testid="rail-identity">
    <TaskStatusMenu :row="row" class="shrink-0" />
    <span class="anchor-chip min-w-0 truncate px-1.5 py-px font-mono text-caption" data-testid="task-anchor">{{ rkCommand(task.anchor) }}</span>
    <CopyAnchorButton :anchor="task.anchor" />
  </div>
</template>
