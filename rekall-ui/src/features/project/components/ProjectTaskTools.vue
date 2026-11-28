<script setup lang="ts">
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import ProjectTaskFilterMenu from '@/features/project/components/ProjectTaskFilterMenu.vue'
import ProjectTaskViewMenu from '@/features/project/components/ProjectTaskViewMenu.vue'
import { TASK_FILTER_KEYS } from '@/features/project/model/project-task-list'
import { useProjectTasksStore } from '@/features/project/store/projectTasks.store'

/** The search, the filters and the view menu: what a person adjusts about the list, apart from which project it shows. */
const projectTasks = useProjectTasksStore()
const { query, isNarrowed } = storeToRefs(projectTasks)
</script>

<template>
  <input
    v-model="query"
    type="text"
    autocomplete="off"
    spellcheck="false"
    placeholder="Search title, code or step"
    aria-label="Search the tasks"
    class="field h-8 w-60 rounded-control px-3 text-small text-text"
    data-testid="task-filter"
  />
  <ProjectTaskFilterMenu v-for="key in TASK_FILTER_KEYS" :key="key" :filter-key="key" />
  <AppButton v-if="isNarrowed" variant="ghost" data-testid="task-clear" @click="projectTasks.clearNarrowing">Clear</AppButton>
  <ProjectTaskViewMenu />
</template>
