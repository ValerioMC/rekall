<script setup lang="ts">
import { computed } from 'vue'
import StepSeal from '@/components/console/StepSeal.vue'
import type { Box } from '@/common/diagram/viewport'
import type { TaskStep } from '@/model/catalog'

/**
 * One step on the flow canvas: its number, seal and title, how many times the work has entered
 * it, and whether the flow begins or ends there. The step the work stands on is lit.
 */
const props = defineProps<{
  step: TaskStep
  box: Box
  current: boolean
  selected: boolean
  entry: boolean
  /** The flow has routes and none leaves this step, so reaching it ends the work. */
  terminal: boolean
}>()

defineEmits<{ select: [] }>()

const visits = computed(() => props.step.visits.length)
const number = computed(() => String(props.step.position + 1).padStart(2, '0'))

const plateClass = computed(() => {
  if (props.current) return 'panel-gold'
  if (props.step.state === 'CLAIMED') return 'panel-gold-quiet'
  if (props.step.state === 'DONE') return 'border-border bg-surface opacity-75'
  return 'panel-slate'
})

const label = computed(() => {
  const parts = [`Step ${props.step.position + 1}, ${props.step.title}`, props.step.state.toLowerCase()]
  if (props.current) parts.push('the work stands here')
  parts.push(visits.value === 1 ? 'entered once' : `entered ${visits.value} times`)
  if (props.entry) parts.push('the flow begins here')
  if (props.terminal) parts.push('the flow ends here')
  return parts.join(', ')
})
</script>

<template>
  <button
    type="button"
    class="focus-ring group/node absolute flex cursor-pointer flex-col justify-between rounded-card border px-3 py-2.5 text-left transition-[border-color,box-shadow] duration-(--motion-quick)"
    :class="[plateClass, selected && 'flow-node-selected']"
    :style="{ left: `${box.x}px`, top: `${box.y}px`, width: `${box.width}px`, height: `${box.height}px` }"
    :aria-label="label"
    :aria-pressed="selected"
    :data-current="current || undefined"
    data-testid="flow-node"
    :data-step-id="step.id"
    @click="$emit('select')"
  >
    <span class="flex min-w-0 items-start gap-2">
      <span class="mt-px shrink-0 font-mono text-micro font-semibold tabular-nums text-text-subtle">{{
        number
      }}</span>
      <span class="line-clamp-2 min-w-0 flex-1 text-small font-medium leading-snug text-text">{{
        step.title
      }}</span>
      <span
        class="relative mt-px block size-[18px] shrink-0"
        :class="step.state === 'RUNNING' && 'step-node-running rounded-full'"
        aria-hidden="true"
      >
        <StepSeal :state="step.state" :next="current && step.state === 'OPEN'" />
      </span>
    </span>

    <span class="flex items-center gap-1.5 text-micro">
      <span v-if="entry" class="flow-tag text-anchor">start</span>
      <span v-if="terminal" class="flow-tag text-safe">end</span>
      <span v-if="current" class="flow-tag text-accent" data-testid="flow-node-current">here</span>
      <span
        class="ml-auto font-mono tabular-nums"
        :class="visits > 1 ? 'text-accent' : 'text-text-subtle'"
        data-testid="flow-node-visits"
      >
        <template v-if="visits > 0">×{{ visits }}</template>
        <template v-else>not reached</template>
      </span>
    </span>
  </button>
</template>

<style scoped>
.flow-node-selected {
  box-shadow:
    0 0 0 2px var(--color-canvas),
    0 0 0 3.5px var(--color-anchor);
}

.flow-tag {
  border-radius: 9999px;
  border: 1px solid currentColor;
  padding: 0 6px;
  line-height: 15px;
  font-weight: 600;
  letter-spacing: 0.02em;
  opacity: 0.85;
}
</style>
