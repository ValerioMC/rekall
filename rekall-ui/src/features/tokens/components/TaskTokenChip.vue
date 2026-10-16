<script setup lang="ts">
import { computed } from 'vue'
import TokenTrendChip from '@/features/tokens/components/TokenTrendChip.vue'
import { identityHue } from '@/common/identity'
import { useTokenUsageStore } from '@/features/tokens/store/tokenUsage.store'
import type { Task } from '@/features/catalog/model/catalog'

/** A task's generated tokens and trend, tinted with its project's colour. */
const props = defineProps<{ task: Task }>()

const tokenUsage = useTokenUsageStore()
const taskIds = computed(() => [props.task.id])
const totals = computed(() => tokenUsage.ofTask(props.task.id))
const color = computed(() => identityHue(props.task.projectId).base)
</script>

<template>
  <TokenTrendChip :task-ids="taskIds" :totals="totals" :color="color" subject="task" />
</template>
