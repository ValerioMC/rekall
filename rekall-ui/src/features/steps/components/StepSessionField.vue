<script setup lang="ts">
import { computed } from 'vue'
import SegmentedControl, { type SegmentOption } from '@/components/ui/SegmentedControl.vue'

/** Whether the step opens a fresh Claude session or runs in the one the step before it is in. */
const props = defineProps<{ startsSession: boolean }>()

const emit = defineEmits<{ choose: [startsSession: boolean] }>()

type Choice = 'join' | 'new'

const OPTIONS: readonly SegmentOption<Choice>[] = [
  { value: 'join', label: 'Join the session above' },
  { value: 'new', label: 'New session' }
]

const choice = computed<Choice>({
  get: () => (props.startsSession ? 'new' : 'join'),
  set: (next) => emit('choose', next === 'new')
})
</script>

<template>
  <div class="mb-3" data-testid="step-session">
    <span class="eyebrow mb-1.5 block">Session</span>
    <SegmentedControl v-model="choice" :options="OPTIONS" label="Session of the step" size="sm" />
    <p class="mt-2 text-caption text-text-subtle">Where Claude works on it once it is on the plan.</p>
  </div>
</template>
