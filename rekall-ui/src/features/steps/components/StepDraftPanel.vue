<script setup lang="ts">
import { computed } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppModeToggle from '@/components/ui/AppModeToggle.vue'
import StatusPill from '@/components/ui/StatusPill.vue'
import DossierFrame from '@/features/steps/components/DossierFrame.vue'
import StepDetailEditor from '@/features/steps/components/StepDetailEditor.vue'
import StepRoutesField from '@/features/steps/components/StepRoutesField.vue'
import StepSeal from '@/features/steps/components/StepSeal.vue'
import StepSessionField from '@/features/steps/components/StepSessionField.vue'
import StepSetup from '@/features/steps/components/StepSetup.vue'
import { useStepEditor } from '@/features/steps/composables/useStepEditor'
import type { Task } from '@/features/catalog/model/catalog'
import type { StepNeighbours } from '@/features/steps/lib/step-navigation'
import type { TaskStep } from '@/features/steps/model/task-step'

/**
 * A draft written in the pane beside the list, in the same frame a step is decided in: its title
 * and detail, then the skills, session, checks and routes it will carry. The dock holds Promote,
 * which waits for a detail so a step is never put on the plan empty.
 */
const props = defineProps<{
  step: TaskStep
  task: Task
  taskSteps: readonly TaskStep[]
  /** Place of this draft among the drafts, from 1. */
  number: number
  total: number
  neighbours: StepNeighbours
  promoting: boolean
  /** The detail takes the caret as the panel opens. */
  autofocus: boolean
  /** There is an overview to go back to; a task with only drafts has none. */
  hasOverview: boolean
  maxWidth?: string
}>()

defineEmits<{
  promote: []
  remove: []
  session: [startsSession: boolean]
  select: [step: TaskStep]
  overview: []
}>()

const editor = useStepEditor()
const canPromote = computed(() => editor.body.value.trim().length > 0 && !props.promoting)

function keepATitle(): void {
  if (!editor.title.value.trim()) editor.title.value = props.step.title
}

const ICON_BUTTON =
  'focus-ring grid size-7 cursor-pointer place-items-center rounded-key text-text-subtle transition-colors duration-(--motion-quick) hover:bg-surface-raised hover:text-text disabled:cursor-default disabled:opacity-40 disabled:hover:bg-transparent disabled:hover:text-text-subtle'
</script>

<template>
  <DossierFrame pinned :max-width="maxWidth" aria-label="Draft" data-testid="draft-panel" data-step-state="DRAFT">
    <template #lead><StepSeal state="DRAFT" /></template>

    <template #title>
      <input
        v-model="editor.title.value"
        class="-mx-2 -my-0.5 w-full rounded-control border border-transparent bg-transparent px-2 py-0.5 text-heading font-medium tracking-[-0.015em] text-text outline-none transition-colors duration-(--motion-quick) placeholder:text-text-subtle hover:bg-surface-raised focus:border-accent focus:bg-canvas focus:ring-1 focus:ring-accent"
        placeholder="Name this step"
        :aria-label="`Title of ${step.title}`"
        data-testid="draft-title-field"
        @input="editor.scheduleSave()"
        @blur="keepATitle"
      />
    </template>

    <template #controls>
      <slot name="controls" />
      <button
        type="button"
        :class="ICON_BUTTON"
        :disabled="!neighbours.previous"
        title="Previous draft (K)"
        aria-label="Previous draft"
        data-testid="draft-previous"
        @click="neighbours.previous && $emit('select', neighbours.previous)"
      >
        <svg class="size-4" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <path d="M4 10l4-4 4 4" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </button>
      <button
        type="button"
        :class="ICON_BUTTON"
        :disabled="!neighbours.next"
        title="Next draft (J)"
        aria-label="Next draft"
        data-testid="draft-next"
        @click="neighbours.next && $emit('select', neighbours.next)"
      >
        <svg class="size-4" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <path d="M4 6l4 4 4-4" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </button>
    </template>

    <template #facts>
      <StatusPill tone="neutral" data-testid="draft-state">Draft</StatusPill>
      <span>Not on the plan yet</span>
      <span data-testid="draft-position">Draft {{ number }} of {{ total }}</span>
      <span class="ml-auto flex items-center gap-1.5">
        <AppModeToggle v-model="editor.mode.value" class="mr-1" testid-prefix="draft-detail" />
        <AppButton v-if="hasOverview" variant="quiet" size="sm" title="Back to the overview (Esc)" data-testid="draft-overview" @click="$emit('overview')">
          Overview
          <kbd class="font-mono text-micro text-text-subtle">Esc</kbd>
        </AppButton>
      </span>
    </template>

    <template #main>
      <StepDetailEditor
        bare
        label="Detail"
        :step-title="step.title"
        testid-prefix="draft"
        editor-height="24rem"
        :autofocus="autofocus"
        placeholder="What this step has to do, once it is promoted. Markdown."
        empty-text="Nothing written under this draft yet."
        write-prompt="Write the detail"
        :path-root="task.projectRepoFolder"
      />
      <StepSetup :step="step" :task="task">
        <StepSessionField :starts-session="step.startsSession" @choose="$emit('session', $event)" />
        <div v-if="step.links.length" class="col-span-full border-t border-border pt-4">
          <StepRoutesField :step="step" :task-steps="taskSteps" />
        </div>
      </StepSetup>
    </template>

    <template #dock>
      <p class="mr-auto text-caption text-text-subtle" data-testid="draft-promote-hint">
        {{ canPromote ? 'Promoting puts it on the plan.' : 'Write the detail to promote it.' }}
      </p>
      <AppButton variant="quiet" data-testid="draft-delete" @click="$emit('remove')">Delete draft</AppButton>
      <AppButton variant="primary" :disabled="!canPromote" :loading="promoting" data-testid="draft-promote" @click="$emit('promote')">
        Promote to the plan
      </AppButton>
    </template>
  </DossierFrame>
</template>
