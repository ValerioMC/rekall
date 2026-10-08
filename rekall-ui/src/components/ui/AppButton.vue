<script setup lang="ts">
import { computed } from 'vue'

type Variant = 'primary' | 'secondary' | 'quiet' | 'ghost' | 'danger' | 'danger-quiet'
type Size = 'sm' | 'md' | 'icon'

const props = withDefaults(
  defineProps<{
    variant?: Variant
    size?: Size
    disabled?: boolean
    loading?: boolean
    /** Set only on a toggle: the key then sinks and lights while on, and says so to assistive tech. */
    pressed?: boolean
    type?: 'button' | 'submit'
  }>(),
  { variant: 'secondary', size: 'md', disabled: false, loading: false, pressed: undefined, type: 'button' }
)

const emit = defineEmits<{ click: [event: MouseEvent] }>()

const VARIANTS: Readonly<Record<Variant, string>> = {
  primary: 'key-gold font-semibold',
  secondary: 'key-slate',
  // A toolbar key: flat until it is wanted, so a row of them does not shout.
  quiet: 'key-quiet',
  ghost: 'bg-transparent text-text-muted hover:key-slate hover:text-text',
  danger: 'key-danger',
  // A destructive action repeated down a list (one per row): ghost at rest, red only under the
  // pointer, so a column of rows is not a column of alarms. The confirm it opens is the guard.
  'danger-quiet': 'bg-transparent text-text-muted hover:bg-danger-soft hover:text-danger'
}

const SIZES: Readonly<Record<Size, string>> = {
  sm: 'h-7 px-3 text-caption font-medium gap-1.5',
  md: 'h-9 px-3.5 text-body gap-2',
  icon: 'size-8 shrink-0'
}

const classes = computed(() => [
  'focus-ring inline-flex items-center justify-center',
  'rounded-control border-0 select-none whitespace-nowrap',
  'disabled:opacity-40 disabled:cursor-not-allowed disabled:active:translate-y-0',
  props.pressed ? 'key-lit' : VARIANTS[props.variant],
  SIZES[props.size]
])
</script>

<template>
  <button
    :type="props.type"
    :class="classes"
    :disabled="props.disabled || props.loading"
    :aria-busy="props.loading || undefined"
    :aria-pressed="props.pressed"
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
