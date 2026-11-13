<script setup lang="ts">
import { useRouter } from 'vue-router'
import AppButton from '@/components/ui/AppButton.vue'
import { TASK_TAB_LABEL, type TaskTab } from '@/features/task/model/task-tab'
import { useConsoleStore } from '@/stores/console.store'

/** A tab the page does not draw yet: it opens in the console, where that pane still lives. */
const props = defineProps<{ tab: TaskTab }>()

const store = useConsoleStore()
const router = useRouter()

function openInConsole(): void {
  if (props.tab === 'wrapup') store.openWrapup()
  else if (props.tab === 'description') store.openDescription()
  void router.push({ name: 'console' })
}
</script>

<template>
  <div class="grid flex-1 place-items-center px-6 py-12 text-center" data-testid="task-tab-in-console">
    <div class="flex max-w-sm flex-col items-center gap-3">
      <p class="text-body font-medium text-text">{{ TASK_TAB_LABEL[tab] }} is still drawn in the console</p>
      <p class="text-small text-text-muted">This tab moves onto the task page in the next step. Until then it opens beside the old panes.</p>
      <AppButton size="sm" data-testid="open-in-console" @click="openInConsole">Open in the console</AppButton>
    </div>
  </div>
</template>
