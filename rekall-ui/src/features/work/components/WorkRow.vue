<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import CopyAnchorButton from '@/components/console/CopyAnchorButton.vue'
import PlanTube from '@/components/ui/PlanTube.vue'
import StatusPill from '@/components/ui/StatusPill.vue'
import TaskMark from '@/features/catalog/components/TaskMark.vue'
import EpicGlyph from '@/features/catalog/components/epic/EpicGlyph.vue'
import { formatTokens } from '@/common/format/token-count'
import { taskMark } from '@/features/catalog/model/task-mark'
import { isEpic } from '@/features/catalog/model/catalog'
import { ageLabel } from '@/features/work/lib/age-label'
import { WORK_KIND_LABEL, WORK_KIND_TONE, type WorkDensity, type WorkRow } from '@/features/work/model/work-view'
import { useConsoleStore } from '@/stores/console.store'
import { useWorkStore } from '@/features/work/store/work.store'
import WorkPeek from '@/features/work/components/WorkPeek.vue'

const props = defineProps<{
  row: WorkRow
  density: WorkDensity
  now: number
  /** Set on an epic's row: the tasks it gathers, and whether they are folded away. */
  epic?: { readonly members: readonly WorkRow[]; readonly folded: boolean }
  /** Set on a task listed under its epic. */
  nested?: boolean
  /** The epic of a task whose epic is not listed with it. */
  epicTitle?: string | null
}>()

const emit = defineEmits<{ toggleEpic: [] }>()

const work = useWorkStore()
const { steps } = storeToRefs(useConsoleStore())

const task = computed(() => props.row.task)
const mark = computed(() => taskMark(task.value, steps.value, false))
const hasPlan = computed(() => task.value.stepCount > 0)
const needsLabel = computed(() => (props.row.claimedSteps > 0 ? `${props.row.claimedSteps} to review` : 'Review'))
const filed = computed(() => props.row.kind === 'filed')
const peeked = computed(() => work.peekedTaskId === task.value.id)
const memberSummary = computed(() => {
  const members = props.epic?.members ?? []
  const done = members.filter((member) => member.kind === 'filed').length
  const noun = members.length === 1 ? 'task' : 'tasks'
  return done > 0 ? `${members.length} ${noun} · ${done} done` : `${members.length} ${noun}`
})
</script>

<template>
  <div>
  <div
    class="ledger-columns group/row focus-ring relative cursor-pointer rounded-control transition-colors duration-(--motion-quick)"
    :class="[
      density === 'compact' ? 'h-[34px]' : 'h-11',
      filed ? 'text-filed' : 'text-text-muted',
      epic && 'bg-epic-soft/40',
      peeked
        ? 'rounded-b-none bg-surface-raised text-text shadow-[inset_0_0_0_1px_var(--color-border-strong)]'
        : 'hover:bg-surface hover:text-text'
    ]"
    role="button"
    tabindex="0"
    :aria-expanded="peeked"
    data-testid="work-row"
    @click="work.togglePeek(task.id)"
    @keydown.enter.self="work.togglePeek(task.id)"
  >
    <button
      v-if="epic"
      type="button"
      class="focus-ring relative grid size-5 cursor-pointer place-items-center rounded-key text-epic hover:bg-surface-hover"
      :aria-expanded="!epic.folded"
      :aria-label="epic.folded ? `Show the ${memberSummary} of this epic` : 'Fold this epic'"
      data-testid="work-epic-toggle"
      @click.stop="emit('toggleEpic')"
      @keydown.enter.stop
    >
      <svg
        class="size-3 transition-transform duration-(--motion-move)"
        :class="!epic.folded && 'rotate-90'"
        viewBox="0 0 12 12"
        fill="none"
        aria-hidden="true"
      >
        <path d="m4.5 2.5 3.5 3.5-3.5 3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </button>
    <span v-else class="relative grid size-3.5 place-items-center">
      <span v-if="nested" class="absolute -top-[22px] bottom-1/2 left-1/2 w-px bg-epic/35" aria-hidden="true" />
      <TaskMark :state="mark.state" :status="task.status" :accepted="mark.accepted" />
    </span>

    <span class="flex min-w-0 items-center gap-2" :class="[filed ? 'font-normal' : 'font-medium text-text', nested && 'pl-3']">
      <EpicGlyph v-if="epic" :size="15" />
      <span class="min-w-0 truncate" :class="[filed && 'text-text-muted', epic && 'font-semibold']" data-testid="work-row-title">{{ task.title }}</span>
      <span
        v-if="epicTitle"
        class="inline-flex min-w-0 max-w-[40%] shrink items-center gap-1 rounded-full bg-epic-soft px-1.5 py-px text-caption text-epic"
        data-testid="work-row-epic"
        :title="`In the epic ${epicTitle}`"
      >
        <EpicGlyph :size="11" />
        <span class="truncate">{{ epicTitle }}</span>
      </span>
      <CopyAnchorButton
        :anchor="task.anchor"
        class="opacity-0 group-hover/row:opacity-100 focus-visible:opacity-100"
        :class="peeked && 'opacity-100'"
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
      <span v-else-if="epic" class="text-epic" data-testid="work-epic-summary">{{ memberSummary }}</span>
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
  <div
    v-if="peeked"
    class="mb-1.5 rounded-b-control bg-surface-raised px-3 pb-3.5 pl-12 pt-1.5 shadow-[inset_0_0_0_1px_var(--color-border-strong)]"
  >
    <WorkPeek :row="row" />
  </div>
  </div>
</template>
