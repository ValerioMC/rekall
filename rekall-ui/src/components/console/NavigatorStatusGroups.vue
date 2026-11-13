<script setup lang="ts">
/**
 * The navigator across projects: tasks under their status, newest first, live ones first, with the
 * backlog and the filed tasks on shelves at the foot. Each status lists its epics before its tasks,
 * closed until opened, and the tasks of an epic are listed under it and nowhere else.
 */
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import NavigatorTaskRow from '@/components/console/NavigatorTaskRow.vue'
import NavigatorFilingDrawer from '@/components/console/NavigatorFilingDrawer.vue'
import NavigatorEpicBranch from '@/features/catalog/components/epic/NavigatorEpicBranch.vue'
import { branchesWithStatus, partitionTasks } from '@/features/catalog/lib/partition-tasks'
import { useConsoleStore } from '@/stores/console.store'
import {
  SHELVED_TASK_STATUSES,
  TASK_STATUS_COLOR,
  TASK_STATUS_LABEL,
  TASK_STATUS_ORDER,
  TASK_STATUS_RING
} from '@/features/catalog/model/task-status'
import { type Task } from '@/features/catalog/model/catalog'
import type { NavigatorShelves } from '@/composables/useNavigatorShelves'
import type { TaskId } from '@/model/branded'

defineProps<{ shelves: NavigatorShelves; runningTaskIds: ReadonlySet<TaskId>; hasProjects: boolean }>()

defineEmits<{ edit: [task: Task] }>()

const store = useConsoleStore()
const { visibleTasks, selectedTaskId, scopeCompany } = storeToRefs(store)

const partitioned = computed(() => partitionTasks(visibleTasks.value))

const grouped = computed(() =>
  TASK_STATUS_ORDER.filter((status) => !SHELVED_TASK_STATUSES.includes(status))
    .map((status) => ({
      status,
      epics: branchesWithStatus(partitioned.value.epics, status),
      tasks: partitioned.value.active.filter((task) => task.status === status)
    }))
    .filter((group) => group.epics.length > 0 || group.tasks.length > 0)
)

const backlogEpics = computed(() => branchesWithStatus(partitioned.value.epics, 'BACKLOG'))
const filedEpics = computed(() => branchesWithStatus(partitioned.value.epics, 'DONE'))
const backlog = computed(() => partitioned.value.backlog)
const filed = computed(() => partitioned.value.filed)
</script>

<template>
  <div v-for="group in grouped" :key="group.status" class="px-2">
    <div class="flex items-center gap-2.5 px-1.5 pb-1.5 pt-5">
      <span class="relative grid size-3 shrink-0 place-items-center" aria-hidden="true">
        <span class="absolute size-3 rounded-full" :class="TASK_STATUS_RING[group.status]" />
        <span class="relative size-[7px] rounded-full" :class="TASK_STATUS_COLOR[group.status]" />
      </span>
      <span class="section-label flex-1">{{ TASK_STATUS_LABEL[group.status] }}</span>
      <span class="shrink-0 rounded-full bg-surface-raised px-1.5 py-px font-mono text-micro tabular-nums text-text-subtle">
        {{ group.epics.length + group.tasks.length }}
      </span>
    </div>

    <div class="flex flex-col gap-0.5">
      <NavigatorEpicBranch
        v-for="branch in group.epics"
        :key="branch.epic.id"
        :branch="branch"
        :expanded="shelves.openEpics.has(branch.epic.id)"
        :selected-task-id="selectedTaskId"
        :running-task-ids="runningTaskIds"
        :show-context="true"
        :show-company="scopeCompany === null"
        :class="group.tasks.length > 0 && 'mb-1.5'"
        @select="store.selectTask($event)"
        @edit="$emit('edit', $event)"
        @toggle="shelves.toggleEpic(branch.epic.id)"
      />
      <NavigatorTaskRow
        v-for="task in group.tasks"
        :key="task.id"
        :task="task"
        :selected="task.id === selectedTaskId"
        :running="runningTaskIds.has(task.id)"
        :show-context="true"
        :show-company="scopeCompany === null"
        @select="store.selectTask(task.id)"
        @edit="$emit('edit', task)"
      />
    </div>
  </div>

  <div v-if="backlog.length || backlogEpics.length" class="px-2 pt-2">
    <NavigatorFilingDrawer
      kind="backlog"
      :count="backlogEpics.length + backlog.length"
      :open="shelves.backlogOpen"
      @toggle="shelves.toggleListBacklog()"
    >
      <NavigatorEpicBranch
        v-for="branch in backlogEpics"
        :key="branch.epic.id"
        :branch="branch"
        :expanded="shelves.openEpics.has(branch.epic.id)"
        :selected-task-id="selectedTaskId"
        :running-task-ids="runningTaskIds"
        :show-context="true"
        :show-company="scopeCompany === null"
        filed
        @select="store.selectTask($event)"
        @edit="$emit('edit', $event)"
        @toggle="shelves.toggleEpic(branch.epic.id)"
      />
      <NavigatorTaskRow
        v-for="task in backlog"
        :key="task.id"
        :task="task"
        :selected="task.id === selectedTaskId"
        :running="runningTaskIds.has(task.id)"
        :show-context="true"
        :show-company="scopeCompany === null"
        filed
        @select="store.selectTask(task.id)"
        @edit="$emit('edit', task)"
      />
    </NavigatorFilingDrawer>
  </div>

  <div v-if="filed.length || filedEpics.length" class="px-2 pt-2">
    <NavigatorFilingDrawer :count="filedEpics.length + filed.length" :open="shelves.filedOpen" @toggle="shelves.toggleListFiled()">
      <NavigatorEpicBranch
        v-for="branch in filedEpics"
        :key="branch.epic.id"
        :branch="branch"
        :expanded="shelves.openEpics.has(branch.epic.id)"
        :selected-task-id="selectedTaskId"
        :running-task-ids="runningTaskIds"
        :show-context="true"
        :show-company="scopeCompany === null"
        filed
        @select="store.selectTask($event)"
        @edit="$emit('edit', $event)"
        @toggle="shelves.toggleEpic(branch.epic.id)"
      />
      <NavigatorTaskRow
        v-for="task in filed"
        :key="task.id"
        :task="task"
        :selected="task.id === selectedTaskId"
        :running="runningTaskIds.has(task.id)"
        :show-context="true"
        :show-company="scopeCompany === null"
        filed
        @select="store.selectTask(task.id)"
        @edit="$emit('edit', task)"
      />
    </NavigatorFilingDrawer>
  </div>

  <p v-if="!grouped.length && !filed.length && !filedEpics.length && !backlog.length && !backlogEpics.length" class="px-4 py-3 text-small leading-relaxed text-text-subtle">
    <template v-if="!hasProjects">No project here yet. Add one from the picker above, then a task can live in it.</template>
    <template v-else>No task here yet.</template>
  </p>
</template>
