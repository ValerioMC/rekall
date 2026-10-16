<script setup lang="ts">
/**
 * One project in the grouped navigator: a card that folds the project away, and under it the live
 * tasks with the backlog and the filed ones on their own shelves.
 */
import { storeToRefs } from 'pinia'
import NavigatorTaskRow from '@/components/console/NavigatorTaskRow.vue'
import NavigatorFilingDrawer from '@/components/console/NavigatorFilingDrawer.vue'
import ProjectPageLink from '@/features/catalog/components/ProjectPageLink.vue'
import ProjectTokenChip from '@/features/tokens/components/ProjectTokenChip.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import ProjectMark from '@/components/ui/ProjectMark.vue'
import { identityHue } from '@/common/identity'
import { useConsoleStore } from '@/stores/console.store'
import { PROJECT_STATUS_LABEL, type Task } from '@/features/catalog/model/catalog'
import type { ProjectTaskGroup } from '@/features/catalog/lib/project-task-groups'
import type { NavigatorShelves } from '@/composables/useNavigatorShelves'
import type { TaskId } from '@/model/branded'

defineProps<{
  group: ProjectTaskGroup
  shelves: NavigatorShelves
  showCompany: boolean
  runningTaskIds: ReadonlySet<TaskId>
}>()

defineEmits<{ edit: [task: Task] }>()

const store = useConsoleStore()
const { selectedTaskId } = storeToRefs(store)
</script>

<template>
  <div class="px-2 pt-2.5 first:pt-1" :class="group.status !== 'ACTIVE' && 'filed-row'">
    <div
      class="project-header-card flex flex-col rounded-card"
      :style="{ '--project-tint-soft': identityHue(group.projectId).soft }"
    >
      <div class="flex items-stretch">
        <button
          data-testid="project-group-toggle"
          class="focus-ring flex min-w-0 flex-1 items-start gap-2.5 rounded-tl-card py-2.5 pl-2.5 pr-2 text-left"
          :aria-expanded="!shelves.collapsedProjects.has(group.projectId)"
          @click="shelves.toggleCollapsed(group.projectId)"
        >
          <svg
            class="mt-1.5 size-3 shrink-0 text-text-subtle transition-transform"
            :class="!shelves.collapsedProjects.has(group.projectId) && 'rotate-90'"
            viewBox="0 0 24 24"
            fill="none"
            aria-hidden="true"
          >
            <path
              d="M9 6l6 6-6 6"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
          <ProjectMark
            class="mt-px"
            :project-id="group.projectId"
            :icon="group.icon"
            :image-version="group.iconImageVersion"
            :size="24"
          />
          <span class="min-w-0 flex-1">
            <span class="flex items-center gap-2">
              <span class="min-w-0 flex-1 truncate text-body font-semibold tracking-[-0.006em] text-text">
                {{ group.projectTitle }}
              </span>
              <AppBadge
                v-if="group.status !== 'ACTIVE'"
                :tone="group.status === 'DONE' ? 'safe' : 'neutral'"
                dot
              >
                {{ PROJECT_STATUS_LABEL[group.status] }}
              </AppBadge>
              <span
                class="shrink-0 rounded-full bg-surface px-1.5 py-px font-mono text-micro tabular-nums text-text-subtle"
              >
                {{ group.tasks.length }}
              </span>
            </span>
          </span>
        </button>
        <ProjectPageLink :project-id="group.projectId" :project-title="group.projectTitle" />
      </div>
      <div class="-mt-1 flex min-w-0 items-center gap-2 pb-2 pl-[66px] pr-2">
        <span v-if="showCompany" class="min-w-0 truncate text-micro text-text-subtle">{{
          group.companyName
        }}</span>
        <ProjectTokenChip class="ml-auto" :project-id="group.projectId" />
      </div>
    </div>

    <div
      v-show="!shelves.collapsedProjects.has(group.projectId)"
      class="ml-[15px] mt-1 flex flex-col gap-0.5 border-l-2 pl-2"
      :style="{ borderLeftColor: identityHue(group.projectId).line }"
    >
      <NavigatorTaskRow
        v-for="task in group.active"
        :key="task.id"
        :task="task"
        :selected="task.id === selectedTaskId"
        :running="runningTaskIds.has(task.id)"
        :show-context="false"
        :show-company="false"
        @select="store.selectTask(task.id)"
        @edit="$emit('edit', task)"
      />

      <NavigatorFilingDrawer
        v-if="group.backlog.length"
        kind="backlog"
        :count="group.backlog.length"
        :open="shelves.openBacklogs.has(group.projectId)"
        @toggle="shelves.toggleBacklog(group.projectId)"
      >
        <NavigatorTaskRow
          v-for="task in group.backlog"
          :key="task.id"
          :task="task"
          :selected="task.id === selectedTaskId"
          :running="runningTaskIds.has(task.id)"
          :show-context="false"
          :show-company="false"
          filed
          @select="store.selectTask(task.id)"
          @edit="$emit('edit', task)"
        />
      </NavigatorFilingDrawer>

      <NavigatorFilingDrawer
        v-if="group.filed.length"
        :count="group.filed.length"
        :open="shelves.openFiled.has(group.projectId)"
        @toggle="shelves.toggleFiled(group.projectId)"
      >
        <NavigatorTaskRow
          v-for="task in group.filed"
          :key="task.id"
          :task="task"
          :selected="task.id === selectedTaskId"
          :running="runningTaskIds.has(task.id)"
          :show-context="false"
          :show-company="false"
          filed
          @select="store.selectTask(task.id)"
          @edit="$emit('edit', task)"
        />
      </NavigatorFilingDrawer>
    </div>
  </div>
</template>
