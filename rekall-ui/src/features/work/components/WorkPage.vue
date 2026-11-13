<script setup lang="ts">
import { storeToRefs } from 'pinia'
import WorkLedger from '@/features/work/components/WorkLedger.vue'
import WorkToolbar from '@/features/work/components/WorkToolbar.vue'
import { useConsoleStore } from '@/stores/console.store'

defineEmits<{ newTask: [] }>()

const { isLoading } = storeToRefs(useConsoleStore())
</script>

<template>
  <div class="min-h-0 flex-1 overflow-y-auto" data-testid="work-page">
    <div class="mx-auto w-full max-w-[1280px] px-6 pb-16">
      <WorkToolbar @new-task="$emit('newTask')" />
      <div v-if="isLoading" class="flex flex-col gap-2 pt-6" aria-hidden="true">
        <div v-for="row in 6" :key="row" class="skeleton h-11 w-full rounded-control" />
      </div>
      <WorkLedger v-else />
    </div>
  </div>
</template>
