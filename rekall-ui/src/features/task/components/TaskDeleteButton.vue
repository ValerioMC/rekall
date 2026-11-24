<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { taskDeletionBlast, taskDeletionBody } from '@/features/catalog/lib/task-deletion'
import { codedTitle } from '@/features/catalog/model/task-code'
import { useConsoleStore } from '@/stores/console.store'
import type { Task } from '@/features/catalog/model/catalog'

/** Deletes the task after a confirm that names what goes with it, then returns to its project's page unless it sits in a list. The project's task manager loses the issue too. */
const props = withDefaults(defineProps<{ task: Task; leavesToProject?: boolean }>(), { leavesToProject: true })

const store = useConsoleStore()
const router = useRouter()
const { run } = useAsyncAction()
const isConfirming = ref(false)

async function remove(): Promise<void> {
  const deleted = await run(async () => {
    await store.deleteTask(props.task.id)
    return true
  }, `Deleted ${codedTitle(props.task)}`)
  isConfirming.value = false
  if (deleted && props.leavesToProject) await router.push({ name: 'project-detail', params: { id: props.task.projectId, tab: 'tasks' } })
}
</script>

<template>
  <AppButton
    variant="danger-quiet"
    size="icon"
    class="!size-7"
    aria-label="Delete this task"
    title="Delete this task"
    data-testid="task-delete"
    @click="isConfirming = true"
  >
    <svg class="size-3.5" viewBox="0 0 14 14" fill="none" aria-hidden="true">
      <path
        d="M2.5 4h9M5.5 4V2.75h3V4M3.75 4l.5 7.25h5.5l.5-7.25M5.75 6.25v3.25M8.25 6.25v3.25"
        stroke="currentColor"
        stroke-width="1.2"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  </AppButton>
  <Transition name="dialog">
    <AppConfirm
      v-if="isConfirming"
      :title="`Delete ${codedTitle(task)}?`"
      :body="taskDeletionBody(task.trackerIssueKey)"
      :blast="taskDeletionBlast(task)"
      confirm-label="Delete task"
      @cancel="isConfirming = false"
      @confirm="remove"
    />
  </Transition>
</template>
