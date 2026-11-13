<script setup lang="ts">
import { computed } from 'vue'
import { useConsoleStore } from '@/stores/console.store'
import DescriptionPane from '@/features/catalog/components/DescriptionPane.vue'
import EpicMembershipCard from '@/features/catalog/components/epic/EpicMembershipCard.vue'
import EpicPane from '@/features/catalog/components/epic/EpicPane.vue'
import { isEpic, type Task } from '@/features/catalog/model/catalog'

/** The Description tab: the description with its review and rewrite actions, or an epic's tasks and shared description. */
const props = defineProps<{ task: Task }>()

const store = useConsoleStore()
const epic = computed(() => (props.task.epicId ? store.tasks.find((task) => task.id === props.task.epicId) ?? null : null))
</script>

<template>
  <div class="flex min-h-0 min-w-0 flex-1 flex-col">
    <EpicMembershipCard v-if="epic" class="mx-5 mt-3 shrink-0" :epic="epic" @open="store.selectTask(epic.id)" />
    <EpicPane v-if="isEpic(task)" embedded />
    <DescriptionPane v-else embedded />
  </div>
</template>
