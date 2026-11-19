<script setup lang="ts" generic="Choice extends string">
defineProps<{
  label: string
  options: readonly { value: Choice; label: string }[]
  modelValue: Choice
  testIdPrefix: string
  disabled?: boolean
}>()

const emit = defineEmits<{ 'update:modelValue': [value: Choice] }>()
</script>

<template>
  <div class="mt-2 flex flex-wrap gap-1.5" role="radiogroup" :aria-label="label">
    <button
      v-for="option in options"
      :key="option.value"
      type="button"
      role="radio"
      :aria-checked="modelValue === option.value"
      :disabled="disabled"
      class="focus-ring inline-flex items-center rounded-key border px-2.5 py-1 text-caption transition-colors disabled:opacity-60"
      :class="
        modelValue === option.value
          ? 'border-accent bg-accent-soft text-accent'
          : 'border-border text-text-subtle hover:border-border-strong hover:text-text-muted'
      "
      :data-testid="`${testIdPrefix}-${option.value}`"
      @click="emit('update:modelValue', option.value)"
    >
      {{ option.label }}
    </button>
  </div>
</template>
