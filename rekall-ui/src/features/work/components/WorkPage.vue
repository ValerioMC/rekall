<script setup lang="ts">
import TasksSection from '@/features/work/components/TasksSection.vue'
import WorkScopeBar from '@/features/work/components/WorkScopeBar.vue'
import { useWorkLayout } from '@/features/work/composables/useWorkLayout'

const layout = useWorkLayout()
</script>

<template>
  <div v-if="layout === 'board'" class="flex min-h-0 flex-1 flex-col px-8" data-testid="work-page">
    <WorkScopeBar />
    <TasksSection chooses-layout />
  </div>
  <div v-else class="min-h-0 flex-1 overflow-y-auto scroll-gutter-stable" data-testid="work-page">
    <div class="page-column pb-16">
      <WorkScopeBar />
      <TasksSection chooses-layout />
    </div>
  </div>
</template>
