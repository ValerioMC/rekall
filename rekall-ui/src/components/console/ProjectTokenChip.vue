<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import TokenTrendChip from '@/components/console/TokenTrendChip.vue'
import { identityHue } from '@/common/identity'
import { useConsoleStore } from '@/stores/console.store'
import { useTokenUsageStore } from '@/stores/tokenUsage.store'
import type { ProjectId } from '@/model/branded'

/** A project's generated tokens and trend: every task of the project counts, filed ones too. */
const props = defineProps<{ projectId: ProjectId }>()

const { tasks } = storeToRefs(useConsoleStore())
const tokenUsage = useTokenUsageStore()
const taskIds = computed(() =>
  tasks.value.filter((task) => task.projectId === props.projectId).map((task) => task.id)
)
const totals = computed(() => tokenUsage.ofProject(props.projectId, tasks.value))
const color = computed(() => identityHue(props.projectId).base)
</script>

<template>
  <TokenTrendChip :task-ids="taskIds" :totals="totals" :color="color" subject="project" />
</template>
