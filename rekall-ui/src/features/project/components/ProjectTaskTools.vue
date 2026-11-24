<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import ProjectTaskViewMenu from '@/features/project/components/ProjectTaskViewMenu.vue'
import { useProjectTasksStore } from '@/features/project/store/projectTasks.store'
import { useConsoleStore } from '@/stores/console.store'

/** The filter and the view menu: what a person adjusts about the list, apart from which project it shows. */
const { tasks } = storeToRefs(useConsoleStore())
const { query } = storeToRefs(useProjectTasksStore())

const hasQuickTasks = computed(() => tasks.value.some((task) => task.quick))
</script>

<template>
  <input
    v-model="query"
    type="text"
    autocomplete="off"
    spellcheck="false"
    placeholder="Filter by title or anchor"
    aria-label="Filter the tasks"
    class="field h-8 w-60 rounded-control px-3 text-small text-text"
    data-testid="task-filter"
  />
  <ProjectTaskViewMenu :shows-quick-tasks="hasQuickTasks" />
</template>
