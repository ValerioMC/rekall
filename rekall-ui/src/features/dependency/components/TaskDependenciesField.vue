<script setup lang="ts">
import { computed, useId, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useConsoleStore } from '@/stores/console.store'
import { useDependenciesStore } from '@/features/dependency/store/dependencies.store'
import { dependencyStateText } from '@/features/dependency/model/dependency'
import { isEpic, type Task } from '@/features/catalog/model/catalog'
import type { TaskId } from '@/model/branded'

/**
 * The tasks of the same project this one waits for. The run queue does not start the task until
 * every one is accepted. Only the person sets this: a session has no tool for it.
 */
const props = defineProps<{ task: Task }>()

const store = useDependenciesStore()
const { tasks } = storeToRefs(useConsoleStore())
const { run } = useAsyncAction()
const pickerId = useId()

const dependencies = computed(() => store.of(props.task.id))

const candidates = computed<Task[]>(() => {
  const chosen = new Set<TaskId>(dependencies.value.map((dependency) => dependency.taskId))
  return tasks.value
    .filter(
      (candidate) =>
        candidate.projectId === props.task.projectId &&
        candidate.id !== props.task.id &&
        !isEpic(candidate) &&
        !chosen.has(candidate.id)
    )
    .sort((left, right) => left.title.localeCompare(right.title))
})

watch(
  () => props.task.id,
  (taskId) => void run(() => store.load(taskId)),
  { immediate: true }
)

function pick(event: Event): void {
  const select = event.target as HTMLSelectElement
  const dependsOn = select.value as TaskId
  select.value = ''
  if (dependsOn) void run(() => store.add(props.task.id, dependsOn))
}

function remove(dependsOn: TaskId): void {
  void run(() => store.remove(props.task.id, dependsOn))
}
</script>

<template>
  <div v-if="!isEpic(task)" class="flex min-w-0 flex-wrap items-center gap-1.5" data-testid="task-dependencies">
    <span class="eyebrow">Depends on</span>
    <span
      v-for="dependency in dependencies"
      :key="dependency.taskId"
      class="inline-flex min-w-0 items-center gap-1 rounded-control border border-border px-2 py-0.5 text-caption"
      :class="dependency.met ? 'text-text-muted' : 'text-text'"
      :title="`${dependency.anchor}: ${dependencyStateText(dependency)}`"
      data-testid="task-dependency"
    >
      <span class="truncate">{{ dependency.title }}</span>
      <span class="shrink-0 text-text-subtle">{{ dependencyStateText(dependency) }}</span>
      <button
        type="button"
        class="focus-ring grid size-4 shrink-0 place-items-center rounded-control text-text-subtle hover:text-text"
        :aria-label="`Stop depending on ${dependency.title}`"
        data-testid="task-dependency-remove"
        @click="remove(dependency.taskId)"
      >
        <svg class="size-3" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
        </svg>
      </button>
    </span>
    <label :for="pickerId" class="sr-only">Add a task this one depends on</label>
    <select
      :id="pickerId"
      class="focus-ring rounded-control border border-border bg-surface px-2 py-0.5 text-caption text-text-muted"
      :disabled="candidates.length === 0"
      data-testid="task-dependency-picker"
      @change="pick"
    >
      <option value="">Add…</option>
      <option v-for="candidate in candidates" :key="candidate.id" :value="candidate.id">{{ candidate.title }}</option>
    </select>
  </div>
</template>
