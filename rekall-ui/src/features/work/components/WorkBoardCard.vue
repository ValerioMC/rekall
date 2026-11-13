<script setup lang="ts">
import { computed } from 'vue'
import PlanTube from '@/components/ui/PlanTube.vue'
import ProjectMark from '@/components/ui/ProjectMark.vue'
import { formatTokens } from '@/common/format/token-count'
import { ageLabel } from '@/features/work/lib/age-label'
import type { WorkRow } from '@/features/work/model/work-view'
import { useConsoleStore } from '@/stores/console.store'

const props = defineProps<{ row: WorkRow; now: number }>()

defineEmits<{ open: [] }>()

const store = useConsoleStore()
const task = computed(() => props.row.task)
const project = computed(() => store.projects.find((candidate) => candidate.id === task.value.projectId) ?? null)
const needsLabel = computed(() => (props.row.claimedSteps > 0 ? `${props.row.claimedSteps} to review` : 'Review'))
</script>

<template>
  <button
    type="button"
    class="focus-ring grid cursor-pointer gap-2.5 rounded-card border border-border bg-surface-raised px-3 pb-2.5 pt-3 text-left transition-[border-color,transform] duration-(--motion-move) hover:-translate-y-px hover:border-border-strong"
    data-testid="board-card"
    @click="$emit('open')"
  >
    <span class="flex items-center gap-2 text-caption text-text-muted">
      <ProjectMark
        :project-id="task.projectId"
        :icon="project?.icon ?? 'folder'"
        :image-version="project?.iconImageVersion ?? null"
        :size="16"
      />
      <span class="min-w-0 truncate">{{ task.projectTitle }}</span>
      <span class="flex-1" />
      <span>{{ ageLabel(task.updatedAt, now) }}</span>
    </span>
    <span class="line-clamp-2 text-body font-medium leading-snug text-text" data-testid="board-card-title">{{ task.title }}</span>
    <PlanTube
      v-if="task.stepCount > 0"
      size="sm"
      :done="task.stepsDone"
      :claimed="row.claimedSteps"
      :total="task.stepCount"
      :live="row.running"
    />
    <span class="flex items-center gap-3 font-mono text-caption text-text-subtle">
      <span v-if="row.running" class="font-sans font-medium text-session-strong">working</span>
      <span v-else-if="row.needs > 0" class="font-sans font-medium text-accent-strong">{{ needsLabel }}</span>
      <span v-if="task.stepCount > 0">{{ task.stepsDone }}/{{ task.stepCount }}</span>
      <span v-if="task.documentCount > 0">{{ task.documentCount }} notes</span>
      <span v-if="row.tokens > 0">{{ formatTokens(row.tokens) }}</span>
    </span>
  </button>
</template>
