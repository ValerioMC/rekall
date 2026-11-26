<script setup lang="ts">
import { useId } from 'vue'

defineProps<{ label: string; modelValue: boolean }>()

const emit = defineEmits<{ 'update:modelValue': [value: boolean] }>()
const descriptionId = useId()
</script>

<template>
  <div class="flex items-start gap-3 border-t border-border px-5 py-3.5">
    <div class="min-w-0 flex-1">
      <p class="text-body font-medium text-text">{{ label }}</p>
      <p :id="descriptionId" class="mt-1 break-words text-small leading-relaxed text-text-subtle">
        <slot />
      </p>
    </div>
    <button
      type="button"
      role="switch"
      :aria-checked="modelValue"
      :aria-label="label"
      :aria-describedby="descriptionId"
      class="focus-ring mt-0.5 h-5 w-9 shrink-0 rounded-full border transition-colors"
      :class="modelValue ? 'border-danger bg-danger-soft' : 'border-border bg-surface-raised'"
      @click="emit('update:modelValue', !modelValue)"
    >
      <span
        class="block size-3.5 rounded-full transition-transform"
        :class="modelValue ? 'translate-x-[18px] bg-danger' : 'translate-x-[2px] bg-text-subtle'"
      />
    </button>
  </div>
</template>
