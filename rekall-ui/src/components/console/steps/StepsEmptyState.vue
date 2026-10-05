<script setup lang="ts">
import { ref } from 'vue'
import CopyGlyph from '@/components/ui/CopyGlyph.vue'
import { rkPlanCommand } from '@/common/format/rk-command'

/** What a task with no steps shows: why a checklist helps, and the two ways to start one. */
const props = defineProps<{ anchor: string }>()

defineEmits<{ 'draft-first': [] }>()

const GHOST_WIDTHS = [62, 45, 54]
const copiedPlan = ref(false)

async function copyPlan(): Promise<void> {
  await navigator.clipboard?.writeText(rkPlanCommand(props.anchor))
  copiedPlan.value = true
  setTimeout(() => (copiedPlan.value = false), 1400)
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

    <h3 class="mb-1.5 text-[21px] font-semibold leading-tight tracking-[-0.02em] text-text">
      Nothing says what is left
    </h3>
    <p class="text-[13px] leading-relaxed text-text-muted">
      The description says what the task is and the wrapup says what it became. Neither says which
      parts are finished, and reading that out of the two is guesswork. Break the work into steps,
      write the detail of each in markdown, and tick them as you review them: the next session opens
      on the ones still open, in full. A step starts as a draft, off the list, until you promote it.
    </p>
    <button
      class="key-gold rounded-[3px] focus-ring mt-5 px-3.5 py-2 text-[12.5px] font-medium"
      data-testid="write-first-step"
      @click="$emit('draft-first')"
    >
      Draft the first step
    </button>

    <p class="mt-6 text-[12.5px] leading-relaxed text-text-muted">
      Or have Claude propose them: <strong class="font-medium text-text">Plan here</strong> above
      opens a session on it, or give this to one you have. It reads the description, the wrapup and
      the code, and puts each step it suggests on a Drafts shelf here, for you to reword and promote.
      It builds nothing.
    </p>
    <button
      class="anchor-chip focus-ring mt-2 inline-flex items-center gap-2 px-2.5 py-1 text-[11.5px] transition-colors hover:border-anchor"
      :class="copiedPlan && 'flash'"
      :title="copiedPlan ? 'Copied' : `Copy ${rkPlanCommand(anchor)}`"
      data-testid="copy-plan-command"
      @click="copyPlan"
    >
      <span class="opacity-60">/rk</span>
      <span>{{ anchor }} plan</span>
      <CopyGlyph :copied="copiedPlan" />
    </button>
  </div>
</template>
