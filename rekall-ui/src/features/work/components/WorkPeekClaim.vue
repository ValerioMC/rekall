<script setup lang="ts">
import AppButton from '@/components/ui/AppButton.vue'
import StepSeal from '@/features/steps/components/StepSeal.vue'
import type { TaskStepState } from '@/features/steps/model/task-step'

withDefaults(
  defineProps<{ state: TaskStepState; title: string; note: string; acceptLabel?: string; decidable?: boolean }>(),
  { acceptLabel: 'Accept', decidable: true }
)

defineEmits<{ accept: []; sendBack: [] }>()
</script>

<template>
  <div
    class="grid min-h-(--spacing-row) grid-cols-[22px_minmax(0,1fr)_auto_auto] items-center gap-2.5 border-t border-border text-small text-text"
    data-testid="peek-claim"
  >
    <span class="size-5.5"><StepSeal :state="state" /></span>
    <span class="truncate">{{ title }}</span>
    <span class="text-caption text-text-subtle">{{ note }}</span>
    <span v-if="decidable" class="flex gap-1.5">
      <AppButton size="sm" data-testid="peek-send-back" @click="$emit('sendBack')">Send back</AppButton>
      <AppButton size="sm" variant="primary" data-testid="peek-accept" @click="$emit('accept')">{{ acceptLabel }}</AppButton>
    </span>
    <span v-else />
  </div>
</template>
