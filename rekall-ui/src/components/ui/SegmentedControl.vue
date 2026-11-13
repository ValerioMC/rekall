<script setup lang="ts" generic="Value extends string">
export type SegmentOption<Value extends string> = Readonly<{ value: Value; label: string; count?: number }>

/** A short choice between a few named states, one always on, as keys in a well. `sm` sits beside small buttons. */
withDefaults(defineProps<{ options: readonly SegmentOption<Value>[]; label: string; size?: 'sm' | 'md' }>(), {
  size: 'md'
})

const selected = defineModel<Value>({ required: true })
</script>

<template>
  <div
    class="inline-flex w-fit shrink-0 gap-0.5 rounded-control border border-border bg-surface p-0.5"
    :class="size === 'sm' ? 'h-7' : 'h-(--spacing-control)'"
    role="group"
    :aria-label="label"
  >
    <button
      v-for="option in options"
      :key="option.value"
      type="button"
      class="focus-ring inline-flex h-full cursor-pointer items-center gap-1.5 rounded-key px-3 text-small transition-colors duration-(--motion-quick)"
      :class="selected === option.value ? 'bg-surface-raised text-text shadow-key' : 'text-text-muted hover:text-text'"
      :aria-pressed="selected === option.value"
      :data-testid="`segment-${option.value}`"
      @click="selected = option.value"
    >
      {{ option.label }}
      <span
        v-if="option.count !== undefined"
        class="font-mono text-micro tabular-nums"
        :class="selected === option.value ? 'text-accent' : 'text-text-subtle'"
      >
        {{ option.count }}
      </span>
    </button>
  </div>
</template>
