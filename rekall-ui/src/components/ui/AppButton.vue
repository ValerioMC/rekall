<script setup lang="ts">
import { computed } from 'vue'

type Variant = 'primary' | 'secondary' | 'ghost' | 'danger'
type Size = 'sm' | 'md'

const props = withDefaults(
  defineProps<{
    variant?: Variant
    size?: Size
    disabled?: boolean
    loading?: boolean
    type?: 'button' | 'submit'
  }>(),
  { variant: 'secondary', size: 'md', disabled: false, loading: false, type: 'button' }
)

const emit = defineEmits<{ click: [event: MouseEvent] }>()

const VARIANTS: Readonly<Record<Variant, string>> = {
  // The accent is spent on exactly one action per screen, so it keeps meaning something.
  primary:
    'bg-accent text-accent-ink border-accent hover:bg-accent-strong hover:border-accent-strong font-semibold shadow-lift',
  secondary:
    'bg-surface-raised text-text border-border-strong hover:bg-surface-hover hover:border-text-subtle',
  ghost: 'bg-transparent text-text-muted border-transparent hover:bg-surface-raised hover:text-text',
  danger: 'bg-transparent text-danger border-danger/40 hover:bg-danger-soft hover:border-danger'
}

const SIZES: Readonly<Record<Size, string>> = {
  sm: 'h-7 px-2.5 text-xs gap-1.5',
  md: 'h-9 px-3.5 text-[13px] gap-2'
}

const classes = computed(() => [
  'focus-ring inline-flex items-center justify-center rounded-[var(--radius-control)] border',
  'transition-all duration-150 active:translate-y-px select-none whitespace-nowrap',
  'disabled:opacity-40 disabled:cursor-not-allowed disabled:active:translate-y-0',
  VARIANTS[props.variant],
  SIZES[props.size]
])
</script>

<template>
  <button
    :type="props.type"
    :class="classes"
    :disabled="props.disabled || props.loading"
    @click="emit('click', $event)"
  >
    <span
      v-if="props.loading"
      class="size-3.5 animate-spin rounded-full border-2 border-current/30 border-t-current"
      aria-hidden="true"
    />
    <slot />
  </button>
</template>
