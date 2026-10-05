<script setup lang="ts">
import { computed, ref } from 'vue'
import CopyGlyph from '@/components/ui/CopyGlyph.vue'
import LaunchClaudeCodeButton from '@/components/claude/LaunchClaudeCodeButton.vue'
import OpenTerminalButton from '@/components/claude/OpenTerminalButton.vue'
import NotesButton from '@/components/console/NotesButton.vue'
import PlanHereButton from '@/components/console/PlanHereButton.vue'
import { identityHue } from '@/common/identity'
import { rkCommand } from '@/common/format/rk-command'
import type { Task, TaskStep } from '@/model/catalog'
import type { TaskStepId } from '@/model/branded'

/**
 * The top of the checklist: the task, its `/rk` anchor, the session buttons, and the ledger, one
 * segment per step so a long checklist still reads at a glance, with the tally and the view
 * toggles on the same line.
 */
const props = defineProps<{
  task: Task
  checklistSteps: readonly TaskStep[]
  draftCount: number
  currentId: TaskStepId | null
  running: boolean
}>()

const hideDone = defineModel<boolean>('hideDone', { required: true })

defineEmits<{
  'review-first': []
  'show-drafts': []
}>()

const hue = computed(() => identityHue(props.task.projectId))
const done = computed(() => props.checklistSteps.filter((step) => step.done).length)
const claimed = computed(() => props.checklistSteps.filter((step) => step.state === 'CLAIMED').length)
const allDone = computed(() => done.value === props.checklistSteps.length)

function segmentClass(step: TaskStep): string {
  if (step.state === 'DONE') return allDone.value ? 'tube-gold tube-safe' : 'tube-gold'
  if (step.state === 'CLAIMED') return 'ledger-claimed'
  if (step.state === 'RUNNING') return 'ledger-running'
  return step.id === props.currentId ? 'tube-next' : 'bg-transparent'
}

const copied = ref(false)

async function copyAnchor(): Promise<void> {
  await navigator.clipboard?.writeText(rkCommand(props.task.anchor))
  copied.value = true
  setTimeout(() => (copied.value = false), 1400)
}
</script>

<template>
  <div class="relative shrink-0 overflow-hidden border-b border-border">
    <div
      class="texture-grid pointer-events-none absolute inset-0 opacity-60"
      :style="{ '--texture-tint': hue.base }"
      aria-hidden="true"
    />
    <div class="pointer-events-none absolute inset-x-0 top-0 h-px" :style="{ background: hue.line }" aria-hidden="true" />

    <header class="relative flex items-start gap-4 px-5 py-3.5">
      <div class="min-w-0 flex-1">
        <p class="eyebrow flex items-center gap-1.5">
          <span class="h-2.5 w-[3px] shrink-0 rounded-full bg-accent" aria-hidden="true" />
          Steps
        </p>
        <h2 class="flex items-center gap-2 truncate text-[19px] font-semibold tracking-[-0.015em] text-text">
          <span class="truncate">{{ task.title }}</span>
          <span
            v-if="running"
            class="inline-flex shrink-0 items-center gap-1.5 rounded-full bg-accent-soft px-2 py-0.5 text-[10.5px] font-semibold tracking-[0.02em] text-accent"
            data-testid="steps-running-flag"
          >
            <span class="relative grid size-2 place-items-center" aria-hidden="true">
              <span class="absolute inline-flex size-2 animate-ping rounded-full bg-accent/60" />
              <span class="relative inline-flex size-1.5 rounded-full bg-accent" />
            </span>
            running
          </span>
        </h2>
        <button
          class="anchor-chip focus-ring mt-1.5 inline-flex items-center gap-2 px-2.5 py-1 text-[11.5px] transition-colors hover:border-anchor"
          :class="copied && 'flash'"
          :title="copied ? 'Copied' : `Copy ${rkCommand(task.anchor)}`"
          data-testid="copy-steps-anchor"
          @click="copyAnchor"
        >
          <span class="opacity-60">/rk</span>
          <span>{{ task.anchor }}</span>
          <CopyGlyph :copied="copied" />
        </button>
      </div>

      <div class="flex shrink-0 items-center gap-1.5">
        <NotesButton :task-id="task.id" />
        <PlanHereButton :task-id="task.id" :anchor="task.anchor" :folder="task.projectRepoFolder" />
        <OpenTerminalButton :task-id="task.id" :step-id="currentId" :folder="task.projectRepoFolder" />
        <LaunchClaudeCodeButton
          :anchors="task.anchor"
          :folder="task.projectRepoFolder"
          missing-hint="Set this project's folder on its page to open a session from it"
        />
      </div>
    </header>

    <div v-if="checklistSteps.length" class="relative flex items-center gap-3 px-5 pb-3" data-testid="steps-ledger">
      <p class="shrink-0 font-mono text-[12px] font-semibold leading-none tabular-nums" data-testid="steps-count"
      ><span :class="allDone ? 'text-safe' : 'text-accent'">{{ done }}</span><span class="text-text-subtle">/{{ checklistSteps.length }}</span></p>
      <span class="trough flex h-[8px] min-w-0 flex-1 gap-[3px] p-[1.5px]" aria-hidden="true">
        <span
          v-for="step in checklistSteps"
          :key="step.id"
          class="h-full min-w-[6px] flex-1 rounded-full transition-colors duration-300"
          :class="segmentClass(step)"
        />
      </span>
      <div v-if="done > 0 || claimed > 0 || draftCount > 0" class="flex shrink-0 items-center gap-2.5 text-[11px]">
        <button
          v-if="claimed > 0"
          class="focus-ring rounded text-accent underline-offset-2 transition-colors hover:underline"
          data-testid="steps-claimed-count"
          @click="$emit('review-first')"
        >
          {{ claimed }} awaiting review
        </button>
        <button
          v-if="draftCount > 0"
          class="focus-ring inline-flex items-center gap-1 rounded text-text-subtle transition-colors hover:text-text"
          data-testid="steps-draft-count"
          @click="$emit('show-drafts')"
        >
          <span class="size-[7px] rounded-[2px] border border-dashed border-current" aria-hidden="true" />
          {{ draftCount }} in draft
        </button>
        <span
          v-if="(claimed > 0 || draftCount > 0) && (done > 0 || claimed > 0)"
          class="h-3 w-px bg-border-strong"
          aria-hidden="true"
        />
        <button
          v-if="done > 0 || claimed > 0"
          class="focus-ring rounded text-text-subtle transition-colors hover:text-text"
          :aria-pressed="hideDone"
          data-testid="toggle-hide-done"
          @click="hideDone = !hideDone"
        >
          {{ hideDone ? 'Show done' : 'Hide done' }}
        </button>
      </div>
    </div>
  </div>
</template>
