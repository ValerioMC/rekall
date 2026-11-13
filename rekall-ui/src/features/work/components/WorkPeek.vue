<script setup lang="ts">
import { computed, onUnmounted, ref } from 'vue'
import { storeToRefs } from 'pinia'
import CopyAnchorButton from '@/components/console/CopyAnchorButton.vue'
import AppButton from '@/components/ui/AppButton.vue'
import StepSeal from '@/features/steps/components/StepSeal.vue'
import OpenTerminalButton from '@/features/claude/components/OpenTerminalButton.vue'
import WorkPeekClaim from '@/features/work/components/WorkPeekClaim.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useNow } from '@/composables/useNow'
import { useOpenTask } from '@/features/work/composables/useOpenTask'
import { ageLabel } from '@/features/work/lib/age-label'
import { handedInWhole } from '@/features/work/lib/work-rows'
import { rkCommand } from '@/common/format/rk-command'
import { stepNumberOf } from '@/features/deviations/model/deviation'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'
import { useWorkStore } from '@/features/work/store/work.store'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { WorkRow } from '@/features/work/model/work-view'

/**
 * What a ledger row opens to: the task's anchor, up to three claimed steps to accept or send back,
 * the task itself when it was handed in whole, and the way into the task and its session. A step
 * just accepted stays in view a moment with its seal stamped before it leaves.
 */
const props = defineProps<{ row: WorkRow }>()

const MAX_CLAIMS = 3
const MINUTE_MS = 60_000
const NEXT_SHOWN = 2
const SETTLE_MS = 1100

const store = useConsoleStore()
const work = useWorkStore()
const toast = useToastStore()
const openTask = useOpenTask()
const { run } = useAsyncAction()
const { steps } = storeToRefs(store)
const now = useNow(MINUTE_MS)

const task = computed(() => props.row.task)
const checklist = computed(() =>
  steps.value.filter((step) => step.taskId === task.value.id).sort((a, b) => a.position - b.position)
)
const claimed = computed(() => checklist.value.filter((step) => step.state === 'CLAIMED'))
const settled = ref<readonly TaskStep[]>([])
const settleTimers = new Set<number>()

const listed = computed(() => {
  const stillClaimed = new Set(claimed.value.map((step) => step.id))
  const stamped = settled.value.filter((step) => !stillClaimed.has(step.id))
  return [...claimed.value, ...stamped].sort((a, b) => a.position - b.position).slice(0, MAX_CLAIMS)
})
const handedIn = computed(() => handedInWhole(task.value))
const hiddenClaims = computed(() => Math.max(0, claimed.value.length - MAX_CLAIMS))
const upNext = computed(() =>
  checklist.value.filter((step) => step.state === 'RUNNING' || step.state === 'OPEN').slice(0, NEXT_SHOWN)
)
const showsClaims = computed(() => listed.value.length > 0 || handedIn.value)
const facts = computed(() =>
  [task.value.hasWrapup ? 'a wrapup' : '', task.value.documentCount > 0 ? `${task.value.documentCount} notes` : '']
    .filter((fact) => fact !== '')
    .join(' · ')
)

function isStamped(step: TaskStep): boolean {
  return settled.value.some((candidate) => candidate.id === step.id)
}

function claimedAge(step: TaskStep): string {
  return isStamped(step) ? 'accepted' : ageLabel(step.claimedAt ?? step.updatedAt, now.value)
}

function stamp(step: TaskStep): void {
  settled.value = [...settled.value, step]
  const timer = window.setTimeout(() => {
    settled.value = settled.value.filter((candidate) => candidate.id !== step.id)
    settleTimers.delete(timer)
  }, SETTLE_MS)
  settleTimers.add(timer)
}

async function accept(step: TaskStep): Promise<void> {
  const outcome = await run(() => store.acceptStep(step.id))
  if (outcome === null) return
  stamp(step)
  work.linger(task.value.id)
  toast.notify(`Accepted step ${stepNumberOf(step, checklist.value) ?? step.title}`, {
    label: 'Reopen',
    run: () => void run(() => store.reopenStep(step.id))
  })
}

async function sendBack(step: TaskStep): Promise<void> {
  const outcome = await run(() => store.sendBackStep(step.id))
  if (outcome === null) return
  work.linger(task.value.id)
  toast.notify(`Sent step ${stepNumberOf(step, checklist.value) ?? step.title} back to the session`)
}

async function acceptTask(): Promise<void> {
  const outcome = await run(() => store.acceptTask(task.value.id))
  if (outcome === null) return
  work.linger(task.value.id)
  toast.notify(`Accepted “${task.value.title}”`, {
    label: 'Reopen',
    run: () => void run(() => store.sendBackTask(task.value.id))
  })
}

async function sendBackTask(): Promise<void> {
  const outcome = await run(() => store.sendBackTask(task.value.id))
  if (outcome === null) return
  work.linger(task.value.id)
  toast.notify('Sent back to the session')
}

onUnmounted(() => settleTimers.forEach((timer) => window.clearTimeout(timer)))
</script>

<template>
  <div
    class="mb-1.5 rounded-b-control bg-surface-raised px-3 pb-3.5 pl-12 pt-1.5 shadow-[inset_0_0_0_1px_var(--color-border-strong)]"
    data-testid="work-peek"
  >
    <div class="flex flex-wrap items-center gap-2.5 pb-2.5 pt-2 text-small text-text-muted">
      <span class="anchor-chip px-1.5 py-px font-mono text-caption" data-testid="peek-anchor">{{ rkCommand(task.anchor) }}</span>
      <CopyAnchorButton :anchor="task.anchor" />
      <span>{{ facts || 'No wrapup yet' }}</span>
    </div>

    <template v-if="showsClaims">
      <WorkPeekClaim
        v-for="step in listed"
        :key="step.id"
        :state="isStamped(step) ? 'DONE' : 'CLAIMED'"
        :title="step.title"
        :note="claimedAge(step)"
        :decidable="!isStamped(step)"
        @accept="accept(step)"
        @send-back="sendBack(step)"
      />
      <WorkPeekClaim
        v-if="handedIn"
        state="CLAIMED"
        title="Work handed in for your review"
        :note="ageLabel(task.claimedAt ?? task.updatedAt, now)"
        accept-label="Accept task"
        @accept="acceptTask"
        @send-back="sendBackTask"
      />
    </template>
    <template v-else-if="upNext.length > 0">
      <div
        v-for="step in upNext"
        :key="step.id"
        class="grid min-h-[38px] grid-cols-[22px_minmax(0,1fr)_auto] items-center gap-2.5 border-t border-border text-small text-text"
        data-testid="peek-next"
      >
        <span class="size-5.5"><StepSeal :state="step.state" /></span>
        <span class="truncate">{{ step.title }}</span>
        <span class="text-caption text-text-subtle">{{ step.state === 'RUNNING' ? 'working' : 'next' }}</span>
      </div>
    </template>
    <p v-else class="border-t border-border py-2.5 text-small text-text-muted" data-testid="peek-description">
      {{ task.description || 'No description yet.' }}
    </p>

    <div class="flex items-center gap-2 border-t border-border pt-2.5">
      <AppButton size="sm" data-testid="peek-open-task" @click="openTask(task.id)">Open task</AppButton>
      <OpenTerminalButton
        v-if="task.status !== 'DONE'"
        :task-id="task.id"
        :anchor="task.anchor"
        :folder="task.projectRepoFolder"
      />
      <span class="flex-1" />
      <span v-if="hiddenClaims > 0" class="font-mono text-caption text-text-subtle" data-testid="peek-more">
        {{ hiddenClaims }} more to review
      </span>
    </div>
  </div>
</template>
