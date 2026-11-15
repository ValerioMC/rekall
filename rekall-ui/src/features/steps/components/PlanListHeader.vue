<script setup lang="ts">
import AppButton from '@/components/ui/AppButton.vue'
import SegmentedControl, { type SegmentOption } from '@/components/ui/SegmentedControl.vue'

/**
 * The head of the step list: its name and count, and the keys that narrow it (hide the done
 * steps, jump to the next claim, list or flow).
 */
defineProps<{ total: number; done: number; claimed: number; canShowFlow: boolean }>()

const hideDone = defineModel<boolean>('hideDone', { required: true })
const view = defineModel<'list' | 'flow'>('view', { required: true })

defineEmits<{ 'review-next': [] }>()

const VIEWS: readonly SegmentOption<'list' | 'flow'>[] = [
  { value: 'list', label: 'List' },
  { value: 'flow', label: 'Flow' }
]
</script>

<template>
  <div class="flex h-11 shrink-0 items-center gap-2 px-4" data-testid="plan-list-header">
    <h2 class="text-title font-semibold text-text">Steps</h2>
    <span class="font-mono text-caption tabular-nums text-text-subtle">{{ total }}</span>
    <span class="flex-1" />
    <AppButton
      v-if="done > 0 || claimed > 0"
      variant="quiet"
      size="sm"
      :aria-pressed="hideDone"
      data-testid="toggle-hide-done"
      @click="hideDone = !hideDone"
    >
      {{ hideDone ? 'Show done' : 'Hide done' }}
    </AppButton>
    <AppButton v-if="claimed > 0" size="sm" data-testid="plan-review-next" @click="$emit('review-next')">
      Review next
    </AppButton>
    <SegmentedControl v-if="canShowFlow" v-model="view" :options="VIEWS" label="Show the steps as" size="sm" />
  </div>
</template>
