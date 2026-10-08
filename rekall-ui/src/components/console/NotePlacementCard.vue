<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import NotePlacedTaskRow from '@/components/console/NotePlacedTaskRow.vue'
import ProjectMark from '@/components/ui/ProjectMark.vue'
import { identityHue } from '@/common/identity'
import { useConsoleStore } from '@/stores/console.store'
import type { ProjectGroup } from '@/common/catalog/task-search'
import type { Task } from '@/model/catalog'
import type { TaskId } from '@/model/branded'

/** One project the note reaches: its mark and name as the card's head, the note's tasks in it below. */
const props = defineProps<{
  group: ProjectGroup<Task>
  showCompany: boolean
  busyTaskId: TaskId | null
}>()

defineEmits<{ detach: [task: Task]; open: [task: Task] }>()

const { projects } = storeToRefs(useConsoleStore())

const project = computed(() => projects.value.find((candidate) => candidate.id === props.group.projectId))
const hue = computed(() => identityHue(props.group.projectId))
</script>

<template>
  <article
    class="placement-card overflow-hidden rounded-card border border-border bg-surface-raised"
    :style="{ '--project-tint': hue.base, '--project-tint-soft': hue.soft }"
    data-testid="note-placement-card"
  >
    <header class="placement-card-head flex items-center gap-2.5 border-b border-border px-3 py-2">
      <ProjectMark
        :project-id="group.projectId"
        :icon="project?.icon ?? 'folder'"
        :image-version="project?.iconImageVersion ?? null"
        :size="22"
      />
      <span class="min-w-0 flex-1">
        <span class="block truncate text-small font-semibold text-text">{{ group.projectTitle }}</span>
        <span v-if="showCompany" class="block truncate text-micro text-text-subtle">{{ group.companyName }}</span>
      </span>
      <span class="shrink-0 rounded-full bg-surface px-1.5 font-mono text-micro tabular-nums leading-4 text-text-subtle">
        {{ group.rows.length }}
      </span>
    </header>

    <ul>
      <NotePlacedTaskRow
        v-for="task in group.rows"
        :key="task.id"
        :task="task"
        :busy="busyTaskId === task.id"
        @detach="$emit('detach', task)"
        @open="$emit('open', task)"
      />
    </ul>
  </article>
</template>

<style scoped>
/* The project's own hue lights the head from its corner, the same tint its navigator card wears. */
.placement-card-head {
  background: radial-gradient(180px 56px at 12px -6px, var(--project-tint-soft), transparent 72%);
  box-shadow: inset 0 2px 0 -1px var(--project-tint);
}
</style>
