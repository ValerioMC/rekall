<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'

/** Hides the quick tasks from the lists, so small changes do not crowd the work that needs a review. */
const store = useConsoleStore()
const { hideQuickTasks } = storeToRefs(store)
</script>

<template>
  <label class="flex cursor-pointer items-center gap-2 text-caption text-text-muted" data-testid="hide-quick-tasks">
    <input v-model="hideQuickTasks" type="checkbox" class="focus-ring size-3.5 accent-(--color-accent)" />
    Hide quick tasks
  </label>
</template>
