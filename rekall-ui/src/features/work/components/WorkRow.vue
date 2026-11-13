<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import CopyAnchorButton from '@/components/console/CopyAnchorButton.vue'
import PlanTube from '@/components/ui/PlanTube.vue'
import StatusPill from '@/components/ui/StatusPill.vue'
import TaskMark from '@/features/catalog/components/TaskMark.vue'
import { formatTokens } from '@/common/format/token-count'
import { taskMark } from '@/features/catalog/model/task-mark'
import { isEpic } from '@/features/catalog/model/catalog'
import { ageLabel } from '@/features/work/lib/age-label'
import { WORK_KIND_LABEL, WORK_KIND_TONE, type WorkDensity, type WorkRow } from '@/features/work/model/work-view'
import { useConsoleStore } from '@/stores/console.store'

const props = defineProps<{ row: WorkRow; density: WorkDensity; now: number }>()

defineEmits<{ open: [] }>()

const { steps } = storeToRefs(useConsoleStore())

const task = computed(() => props.row.task)
const mark = computed(() => taskMark(task.value, steps.value, false))
const hasPlan = computed(() => task.value.stepCount > 0)
const needsLabel = computed(() => (props.row.claimedSteps > 0 ? `${props.row.claimedSteps} to review` : 'Review'))
const filed = computed(() => props.row.kind === 'filed')
</script>

<template>
  <div
    class="ledger-columns group/row focus-ring relative cursor-pointer rounded-control transition-colors duration-(--motion-quick) hover:bg-surface hover:text-text"
    :class="[density === 'compact' ? 'h-[34px]' : 'h-11', filed ? 'text-filed' : 'text-text-muted']"
    role="button"
    tabindex="0"
    data-testid="work-row"
    @click="$emit('open')"
    @keydown.enter.self="$emit('open')"
  >
    <span class="relative grid size-3.5 place-items-center">
      <TaskMark :state="mark.state" :status="task.status" :accepted="mark.accepted" />
    </span>

    <span class="flex min-w-0 items-center gap-2" :class="filed ? 'font-normal' : 'font-medium text-text'">
      <span class="min-w-0 truncate" :class="filed && 'text-text-muted'" data-testid="work-row-title">{{ task.title }}</span>
      <CopyAnchorButton
        :anchor="task.anchor"
        class="opacity-0 group-hover/row:opacity-100 focus-visible:opacity-100"
        @click.stop
      />
    </span>

    <span>
      <StatusPill :tone="WORK_KIND_TONE[row.kind]" :live="row.running">{{ WORK_KIND_LABEL[row.kind] }}</StatusPill>
    </span>

    <span class="flex items-center gap-2.5 font-mono text-caption" data-testid="work-row-plan">
      <template v-if="hasPlan">
        <PlanTube class="w-26" :done="task.stepsDone" :claimed="row.claimedSteps" :total="task.stepCount" :live="row.running" />
        {{ task.stepsDone }}/{{ task.stepCount }}
      </template>
      <span v-else class="text-text-subtle">{{ isEpic(task) ? 'epic' : 'no steps' }}</span>
    </span>

    <span class="text-small font-medium" data-testid="work-row-needs">
      <span v-if="row.running" class="text-session-strong">working</span>
      <span v-else-if="row.needs > 0" class="text-accent-strong">{{ needsLabel }}</span>
      <span v-else class="text-text-subtle" aria-hidden="true">·</span>
    </span>

    <span class="text-right font-mono text-caption tabular-nums text-text-subtle max-[1180px]:hidden">
      {{ task.documentCount || '' }}
    </span>
    <span class="text-right font-mono text-caption tabular-nums text-text-subtle max-[1180px]:hidden">
      {{ row.tokens > 0 ? formatTokens(row.tokens) : '' }}
    </span>
    <span class="text-right font-mono text-caption tabular-nums text-text-subtle">
      {{ ageLabel(task.updatedAt, now) }}
    </span>
  </div>
</template>
