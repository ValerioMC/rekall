<script setup lang="ts" generic="Value extends string">
export type SegmentOption<Value extends string> = Readonly<{ value: Value; label: string }>

/** A short choice between a few named states, one always on, as keys in a well. */
defineProps<{ options: readonly SegmentOption<Value>[]; label: string }>()

const selected = defineModel<Value>({ required: true })
</script>

<template>
  <div class="inline-flex gap-0.5 rounded-control border border-border bg-surface p-0.5" role="group" :aria-label="label">
    <button
      v-for="option in options"
      :key="option.value"
      type="button"
      class="focus-ring inline-flex h-7 cursor-pointer items-center gap-1.5 rounded-control px-3 text-small transition-colors duration-(--motion-quick)"
      :class="selected === option.value ? 'bg-surface-raised text-text shadow-key' : 'text-text-muted hover:text-text'"
      :aria-pressed="selected === option.value"
      :data-testid="`segment-${option.value}`"
      @click="selected = option.value"
    >
      {{ option.label }}
    </button>
  </div>
</template>
