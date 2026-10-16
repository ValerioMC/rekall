<script setup lang="ts">
import { computed, toRef } from 'vue'
import CommitReferenceList from '@/features/commits/components/CommitReferenceList.vue'
import StepPassHistory from '@/features/steps/components/StepPassHistory.vue'
import StepSeal from '@/features/steps/components/StepSeal.vue'
import SkillMark from '@/features/skills/components/SkillMark.vue'
import TokenCount from '@/components/ui/TokenCount.vue'
import { useStepEditor } from '@/features/steps/composables/useStepEditor'
import { useStepSkills } from '@/features/skills/composables/useStepSkills'
import { useConsoleStore } from '@/stores/console.store'
import { useTokenUsageStore } from '@/features/tokens/store/tokenUsage.store'
import { relativeTime } from '@/common/format/relative-time'
import { stepIsComplete, type Task, type TaskStep } from '@/features/catalog/model/catalog'
import StepDetailEditor from './StepDetailEditor.vue'
import StepRowControls from './StepRowControls.vue'
import StepSkillsField from './StepSkillsField.vue'
import StepStateBar from './StepStateBar.vue'

/**
 * One step on the checklist: its node on the rail, its title and state, its controls, and, when
 * it is the open one, its state bar, commits, sealed passes and detail.
 */
const props = defineProps<{
  step: TaskStep
  task: Task
  /** Where the rail runs beside this row, and whether that stretch is finished work. */
  railStyle: Record<string, string>
  railSpent: boolean
  isNext: boolean
  allDone: boolean
  isLast: boolean
  reopenArmed: boolean
}>()

defineEmits<{
  'mark-forward': []
  'to-draft': []
  move: [by: number]
  remove: []
  'send-back': []
  reopen: []
}>()

const store = useConsoleStore()
const { expandedId, toggle } = useStepEditor()

const expanded = computed(() => expandedId.value === props.step.id)
const commits = computed(() => store.stepCommitReferences(props.step.id))
const tokenUsage = useTokenUsageStore()
const tokens = computed(() => tokenUsage.ofStep(props.step.id))
const { effective: skills } = useStepSkills(toRef(props, 'step'), toRef(props, 'task'))

const checkboxLabel = computed(() => {
  if (props.step.state === 'DONE') return `${props.step.title} is done. Reopen it from its detail.`
  if (props.step.state === 'CLAIMED') return `Accept ${props.step.title}`
  return `Mark ${props.step.title} done`
})

const ariaChecked = computed(() => {
  if (props.step.state === 'DONE') return 'true'
  if (props.step.state === 'CLAIMED') return 'mixed'
  return 'false'
})

const panelClass = computed(() => {
  if (props.step.state === 'RUNNING') return 'panel-gold'
  if (props.step.state === 'CLAIMED') return 'panel-gold-quiet'
  if (props.isNext) return 'panel-slate'
  if (props.step.state === 'DONE') return 'border-transparent opacity-60'
  return 'border-transparent transition-[border-color] duration-(--motion-quick) group-hover/step:border-border'
})
</script>

<template>
  <li
    class="group/step relative min-w-0 pb-1.5 pl-9"
    data-testid="step-row"
    :data-step-id="step.id"
    :data-step-state="step.state"
  >
    <span
      class="absolute left-[11px] w-px -translate-x-1/2"
      :class="railSpent ? 'spent-rail' : 'pending-rail'"
      :style="railStyle"
      aria-hidden="true"
    />

    <button
      class="focus-ring hit-target absolute left-0 top-[7px] z-10 size-[22px] rounded-full"
      :class="{
        'cursor-default': step.state === 'DONE',
        'step-node-running': step.state === 'RUNNING',
        'shadow-[0_0_0_4px_var(--color-accent-soft)]': step.state === 'RUNNING' || isNext
      }"
      role="checkbox"
      :aria-checked="ariaChecked"
      :aria-label="checkboxLabel"
      data-testid="step-checkbox"
      data-seal-host
      @click="$emit('mark-forward')"
    >
      <StepSeal :state="step.state" :next="isNext" :complete="allDone" />
    </button>

    <div class="min-w-0 rounded-card border px-3 py-2" :class="panelClass">
      <div class="flex items-start gap-2">
        <button
          class="focus-ring min-w-0 flex-1 rounded-key text-left"
          :aria-expanded="expanded"
          data-testid="step-title"
          @click="toggle(step)"
        >
          <span
            class="block text-body leading-snug transition-colors"
            :class="step.state === 'DONE' ? 'text-text-subtle line-through decoration-text-subtle/50' : 'text-text'"
          >
            {{ step.title }}
          </span>
          <span class="mt-1 flex flex-wrap items-center gap-2 text-micro">
            <span
              v-if="step.state === 'RUNNING'"
              class="inline-flex items-center gap-1 rounded-full bg-accent px-1.5 py-px text-micro font-semibold tracking-[0.02em] text-accent-ink"
              data-testid="step-running-badge"
            >
              Running
            </span>
            <span
              v-else-if="step.state === 'CLAIMED'"
              class="rounded-full border border-accent/40 px-1.5 py-px text-micro font-semibold tracking-[0.02em] text-accent"
              data-testid="step-claimed-badge"
            >
              Awaiting review
            </span>
            <span
              v-else-if="isNext"
              class="rounded-full bg-accent-soft px-1.5 py-px text-micro font-semibold tracking-[0.02em] text-accent"
            >
              Next
            </span>
            <span v-if="step.state === 'CLAIMED' && step.claimedAt" class="text-text-subtle">
              claimed {{ relativeTime(step.claimedAt) }}
            </span>
            <span v-if="step.state === 'DONE' && step.doneAt" class="text-text-subtle">
              done {{ relativeTime(step.doneAt) }}
            </span>
            <span v-if="step.bodyMarkdown?.trim()" class="flex items-center gap-1 text-text-subtle" data-testid="step-has-detail">
              <svg class="size-2.5" viewBox="0 0 12 12" fill="none" aria-hidden="true">
                <path d="M1.6 3h8.8M1.6 6h8.8M1.6 9h5.4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
              </svg>
              detail
            </span>
            <span v-else-if="!stepIsComplete(step.state)" class="text-text-subtle">no detail yet</span>
            <SkillMark :skills="skills" />
            <TokenCount :totals="tokens" />
          </span>
        </button>

        <div class="flex shrink-0 items-center gap-0.5 opacity-0 focus-within:opacity-100 group-hover/step:opacity-100">
          <button
            v-if="step.state === 'OPEN'"
            class="focus-ring mr-0.5 grid h-6 place-items-center rounded-key px-1.5 text-micro font-medium text-text-subtle transition-colors hover:bg-surface-hover hover:text-text"
            :aria-label="`Return ${step.title} to draft`"
            data-testid="step-to-draft"
            @click="$emit('to-draft')"
          >
            To draft
          </button>
          <StepRowControls
            :title="step.title"
            :can-move-up="step.position > 0"
            :can-move-down="!isLast"
            testid-prefix="step"
            @up="$emit('move', -1)"
            @down="$emit('move', 1)"
            @remove="$emit('remove')"
          />
        </div>
      </div>

      <Transition name="detail">
        <div v-if="expanded" class="mt-2.5" data-testid="step-detail">
          <StepStateBar
            :step="step"
            :task="task"
            :reopen-armed="reopenArmed"
            @accept="$emit('mark-forward')"
            @send-back="$emit('send-back')"
            @reopen="$emit('reopen')"
          />

          <div v-if="commits.length" class="mb-3" data-testid="step-commits">
            <span class="eyebrow">Commits <span class="font-mono">{{ commits.length }}</span></span>
            <CommitReferenceList :references="commits" dense />
          </div>

          <StepPassHistory v-if="step.passes.length" :passes="step.passes" class="mb-3" />

          <StepSkillsField v-if="!stepIsComplete(step.state)" :step="step" :task="task" />

          <StepDetailEditor
            :step-title="step.title"
            testid-prefix="step"
            :label="step.passes.length ? `Feedback for pass ${step.passes.length + 1}` : 'Detail'"
            editor-height="300px"
            :placeholder="
              step.passes.length
                ? 'What the next pass has to change. Claude reads it after the sealed passes above.'
                : 'What this step has to do. Markdown, and Claude gets it while the step is open.'
            "
            :empty-text="step.passes.length ? 'No feedback yet.' : 'Nothing written under this step yet.'"
            :write-prompt="step.passes.length ? 'Write what the next pass has to change' : 'Write the detail'"
            :path-root="task.projectRepoFolder"
          />
        </div>
      </Transition>
    </div>
  </li>
</template>
