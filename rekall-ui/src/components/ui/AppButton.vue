<script setup lang="ts">
import { computed } from 'vue'

type Variant = 'primary' | 'secondary' | 'ghost' | 'danger' | 'danger-quiet'
type Size = 'sm' | 'md'

const props = withDefaults(
  defineProps<{
    variant?: Variant
    size?: Size
    disabled?: boolean
    loading?: boolean
    /** Square-cornered keycap instead of the polished metal key: primary and secondary only. */
    square?: boolean
    type?: 'button' | 'submit'
  }>(),
  { variant: 'secondary', size: 'md', disabled: false, loading: false, square: false, type: 'button' }
)

const emit = defineEmits<{ click: [event: MouseEvent] }>()

const VARIANTS: Readonly<Record<Variant, string>> = {
  primary: 'metal-gold font-semibold',
  secondary: 'metal-slate',
  ghost:
    'bg-transparent text-text-muted border-transparent hover:metal-slate hover:text-text active:shadow-[var(--shadow-well)]',
  danger:
    'metal-slate text-danger border-danger/40 hover:bg-danger-soft hover:border-danger hover:text-danger',
  // A destructive action repeated down a list (one per row): ghost at rest, red only under the
  // pointer, so a column of rows is not a column of alarms. The confirm it opens is the guard.
  'danger-quiet':
    'bg-transparent text-text-muted border-transparent hover:bg-danger-soft hover:text-danger'
}

const KEYCAPS: Readonly<Partial<Record<Variant, string>>> = {
  primary: 'key-gold font-semibold',
  secondary: 'key-slate'
}

const SIZES: Readonly<Record<Size, string>> = {
  sm: 'h-7 px-2.5 text-xs gap-1.5',
  md: 'h-9 px-3.5 text-[13px] gap-2'
}

const keycap = computed(() => (props.square ? KEYCAPS[props.variant] : undefined))

const classes = computed(() => [
  'focus-ring inline-flex items-center justify-center',
  keycap.value ? 'rounded-[3px] border-0' : 'rounded-[var(--radius-control)] border active:translate-y-px',
  'select-none whitespace-nowrap',
  'disabled:opacity-40 disabled:cursor-not-allowed disabled:active:translate-y-0',
  keycap.value ?? VARIANTS[props.variant],
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
      class="size-3.5 animate-spin rounded-full border-2 border-current/25 border-t-current drop-shadow-[0_0_3px_currentColor]"
      aria-hidden="true"
    />
    <slot />
  </button>
</template>
