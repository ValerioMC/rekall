<script setup lang="ts">
import AppButton from '@/components/ui/AppButton.vue'
import { ref } from 'vue'
import CopyGlyph from '@/components/ui/CopyGlyph.vue'
import { rkPlanCommand, rkPlanFlowCommand } from '@/common/format/rk-command'

/** What a task with no steps shows: why a checklist helps, and the two ways to start one. */
const props = defineProps<{ anchor: string }>()

defineEmits<{ 'draft-first': [] }>()

type PlanChip = 'plan' | 'flow'

const GHOST_WIDTHS = [62, 45, 54]
const PLAN_CHIPS: readonly PlanChip[] = ['plan', 'flow']

const copied = ref<PlanChip | null>(null)

function planLine(chip: PlanChip): string {
  return chip === 'flow' ? rkPlanFlowCommand(props.anchor) : rkPlanCommand(props.anchor)
}

async function copyPlan(chip: PlanChip): Promise<void> {
  await navigator.clipboard?.writeText(planLine(chip))
  copied.value = chip
  setTimeout(() => (copied.value = null), 1400)
}
</script>

<template>
  <div class="max-w-[560px] py-4">
    <div class="relative mb-7" aria-hidden="true">
      <span class="absolute bottom-[18px] left-[11px] top-[18px] w-px -translate-x-1/2 bg-border-strong" />
      <div v-for="width in GHOST_WIDTHS" :key="width" class="relative flex h-9 items-center pl-9">
        <span
          class="absolute left-0 top-1/2 size-[22px] -translate-y-1/2 rounded-full border border-dashed border-border-strong bg-canvas"
        />
        <span class="h-[7px] rounded-full bg-border" :style="{ width: `${width}%` }" />
      </div>
    </div>

    <h3 class="mb-1.5 text-heading font-semibold leading-tight tracking-[-0.02em] text-text">
      Nothing says what is left
    </h3>
    <p class="text-body leading-relaxed text-text-muted">
      The description says what the task is and the wrapup says what it became. Neither says which
      parts are finished, and reading that out of the two is guesswork. Break the work into steps,
      write the detail of each in markdown, and tick them as you review them: the next session opens
      on the ones still open, in full. A step starts as a draft, off the list, until you promote it.
    </p>
    <AppButton
      variant="primary"
      class="mt-5"
      data-testid="write-first-step"
      @click="$emit('draft-first')"
    >
      Draft the first step
    </AppButton>

    <p class="mt-6 text-small leading-relaxed text-text-muted">
      Or have Claude propose them: <strong class="font-medium text-text">Plan here</strong> above
      opens a session on it, or give this to one you have. It reads the description, the wrapup and
      the code, and puts each step it suggests on a Drafts shelf here, for you to reword and promote.
      It builds nothing. When what a step finds decides what comes next (tests that can fail, a
      review that can reject), <strong class="font-medium text-text">Plan a flow</strong> has it
      propose the steps and the routes between them instead.
    </p>
    <div class="mt-2 flex flex-wrap gap-2">
      <button
        v-for="chip in PLAN_CHIPS"
        :key="chip"
        class="anchor-chip focus-ring inline-flex items-center gap-2 px-2.5 py-1 text-caption transition-colors hover:border-anchor"
        :class="copied === chip && 'flash'"
        :title="copied === chip ? 'Copied' : `Copy ${planLine(chip)}`"
        :data-testid="chip === 'flow' ? 'copy-plan-flow-command' : 'copy-plan-command'"
        @click="copyPlan(chip)"
      >
        <span class="opacity-60">/rk</span>
        <span>{{ anchor }} {{ chip === 'flow' ? 'plan flow' : 'plan' }}</span>
        <CopyGlyph :copied="copied === chip" />
      </button>
    </div>
  </div>
</template>
