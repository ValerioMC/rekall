<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { TaskStep } from '@/features/catalog/model/catalog'

/**
 * How many times the flow may enter one step. A session moving into a step whose budget is spent
 * is refused and stops, so a loop running unattended cannot go round forever.
 */
const props = defineProps<{
  step: TaskStep
  busy: boolean
}>()

const emit = defineEmits<{ limit: [maxVisits: number | null] }>()

const MOST = 99

const typed = ref('')
watch(
  () => props.step.maxVisits,
  (max) => (typed.value = max === null ? '' : String(max)),
  { immediate: true }
)

const entered = computed(() => props.step.visits.length)
const spent = computed(() => props.step.maxVisits !== null && entered.value >= props.step.maxVisits)

/** The typed budget, kept only when it is a whole number in range; anything else snaps back. */
function commit(): void {
  const text = typed.value.trim()
  const wanted = text === '' ? null : Number(text)
  const valid = wanted === null || (Number.isInteger(wanted) && wanted >= 1 && wanted <= MOST)
  if (!valid || wanted === props.step.maxVisits) {
    typed.value = props.step.maxVisits === null ? '' : String(props.step.maxVisits)
    return
  }
  emit('limit', wanted)
}
</script>

<template>
  <section aria-labelledby="flow-visit-budget" data-testid="flow-visit-budget">
    <h4 id="flow-visit-budget" class="eyebrow mb-2">Visit budget</h4>
    <div class="flex items-center gap-2">
      <input
        v-model="typed"
        type="text"
        inputmode="numeric"
        maxlength="2"
        class="field h-8 w-20 rounded-control px-2 font-mono text-caption tabular-nums text-text"
        placeholder="none"
        aria-label="How many times the flow may enter this step"
        :disabled="busy"
        data-testid="flow-visit-budget-input"
        @keydown.enter.prevent="($event.target as HTMLInputElement).blur()"
        @blur="commit"
      />
      <span
        v-if="step.maxVisits !== null"
        class="font-mono text-caption tabular-nums"
        :class="spent ? 'text-danger' : 'text-text-muted'"
        data-testid="flow-visit-budget-used"
      >
        {{ entered }} of {{ step.maxVisits }} used
      </span>
      <span v-else class="text-caption text-text-subtle">no limit</span>
    </div>
    <p class="mt-2 text-micro leading-relaxed text-text-subtle">
      Once the flow has entered this step that many times, a move into it is refused and the session stops to
      report. Raise it or reset the moves to go on.
    </p>
  </section>
</template>
