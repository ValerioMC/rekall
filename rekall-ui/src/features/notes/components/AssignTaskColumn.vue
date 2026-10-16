<script setup lang="ts">
/**
 * The last column of the note-assignment walk: the picked project's tasks, filtered, open ones
 * first by default, each ticked when the note already sits on it. Ticking is the dialog's to save.
 */
import { computed, ref } from 'vue'
import { useSettledOrder } from '@/composables/useSettledOrder'
import { TASK_STATUS_COLOR, TASK_STATUS_LABEL, type Task } from '@/features/catalog/model/catalog'
import type { ProjectId, TaskId } from '@/model/branded'

const props = defineProps<{
  projectId: ProjectId | null
  tasks: readonly Task[]
  attachedIds: ReadonlySet<TaskId>
}>()

defineEmits<{ toggle: [taskId: TaskId] }>()

const filter = defineModel<string>('filter', { required: true })
const showAll = defineModel<boolean>('showAll', { required: true })

const TASK_SCOPES = [
  { value: 'open', label: 'Open' },
  { value: 'all', label: 'All' }
] as const

const filterField = ref<HTMLInputElement | null>(null)

const filteredTasks = computed(() => {
  if (props.projectId === null) return []
  const needle = filter.value.trim().toLowerCase()
  return props.tasks
    .filter((task) => showAll.value || task.status !== 'DONE')
    .filter((task) => !needle || `${task.title} ${task.label}`.toLowerCase().includes(needle))
})

// A tick refetches every task, so rows hold their places until the project or filter changes.
const projectTasks = useSettledOrder(() => filteredTasks.value, (task) => task.id, [
  () => props.projectId,
  filter,
  showAll
])

const doneInProject = computed(() => props.tasks.filter((task) => task.status === 'DONE').length)

defineExpose({ focusFilter: () => filterField.value?.focus() })
</script>

<template>
  <section class="flex min-h-0 min-w-0 flex-1 basis-0 flex-col" aria-label="Tasks">
    <div class="shrink-0 px-3.5 pb-2 pt-3">
      <div class="flex items-center justify-between pb-1.5">
        <p class="eyebrow">Task</p>
        <div v-if="projectId !== null" class="flex items-center gap-2">
          <div
            v-if="doneInProject"
            class="flex gap-0.5 rounded-control bg-canvas p-0.5"
            role="group"
            aria-label="Which tasks to show"
          >
            <button
              v-for="scope in TASK_SCOPES"
              :key="scope.value"
              class="focus-ring hit-target relative h-[18px] rounded-key px-1.5 text-micro transition-colors"
              :class="
                (scope.value === 'all') === showAll
                  ? 'bg-surface-raised text-text'
                  : 'text-text-subtle hover:text-text'
              "
              :aria-pressed="(scope.value === 'all') === showAll"
              :data-testid="`assign-scope-${scope.value}`"
              @click="showAll = scope.value === 'all'"
            >
              {{ scope.label }}
            </button>
          </div>
          <span class="font-mono text-micro text-text-subtle">
            {{ projectTasks.length }}
          </span>
        </div>
      </div>
      <input
        ref="filterField"
        v-model="filter"
        type="search"
        data-testid="assign-task-filter"
        placeholder="Filter tasks"
        :disabled="projectId === null"
        class="field text-text h-8 w-full rounded-control px-2.5 text-small disabled:opacity-40"
      />
    </div>
    <div class="relative min-h-0 flex-1">
      <div
        v-if="projectId !== null && projectTasks.length"
        class="h-full space-y-0.5 overflow-y-auto overscroll-contain px-1.5 py-1"
      >
        <button
          v-for="task in projectTasks"
          :key="task.id"
          data-testid="assign-task"
          class="focus-ring flex w-full items-center gap-2.5 rounded-control px-2.5 py-2 text-left transition-colors"
          :class="
            attachedIds.has(task.id)
              ? 'bg-surface-raised text-text'
              : 'text-text-muted hover:bg-surface-raised hover:text-text'
          "
          :aria-pressed="attachedIds.has(task.id)"
          @click="$emit('toggle', task.id)"
        >
          <span
            class="grid size-4 shrink-0 place-items-center rounded-control border transition-colors"
            :class="
              attachedIds.has(task.id)
                ? 'border-accent-deep bg-accent-soft text-accent'
                : 'border-border-strong text-transparent'
            "
            aria-hidden="true"
          >
            <svg class="size-3" viewBox="0 0 24 24" fill="none">
              <path
                d="M5 12.5l4.5 4.5L19 7.5"
                stroke="currentColor"
                stroke-width="2.6"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
          </span>
          <span class="min-w-0 flex-1">
            <span class="block truncate text-small" :title="task.title">
              {{ task.title }}
            </span>
            <span
              class="anchor-chip mt-0.5 inline-block max-w-full truncate align-bottom px-1.5 py-px text-micro leading-[15px]"
              :title="task.label"
            >
              {{ task.label }}
            </span>
          </span>
          <span
            class="size-[7px] shrink-0 rounded-full"
            :class="TASK_STATUS_COLOR[task.status]"
            :title="TASK_STATUS_LABEL[task.status]"
            aria-hidden="true"
          />
        </button>
      </div>
      <div v-else class="grid h-full place-items-center px-5">
        <p class="text-center text-small leading-relaxed text-text-subtle">
          <template v-if="projectId === null">Pick a project to see its tasks.</template>
          <template v-else-if="filter.trim()">No task matches that filter.</template>
          <template v-else-if="!showAll && doneInProject">
            Every task here is done. Switch to All to see them.
          </template>
          <template v-else>No task in this project yet.</template>
        </p>
      </div>
      <div
        v-if="projectId !== null && projectTasks.length"
        class="pointer-events-none absolute inset-x-1.5 bottom-0 h-5 bg-gradient-to-t from-surface to-transparent"
        aria-hidden="true"
      />
    </div>
  </section>
</template>
