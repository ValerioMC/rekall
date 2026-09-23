<script setup lang="ts">
/**
 * The usage ceiling, set where it will act: one track per usage window the queue watches, each
 * filled to where that window stands now, and one amber waterline across all of them at the
 * ceiling. Dragging the line is setting the ceiling; a window whose fill reaches it turns yellow,
 * because that is the window that will hold the queue. Where the line crosses a track it is a
 * solid notch; between tracks it is a faint dashed thread, so it never strikes through a label.
 *
 * The line is a native range input laid over the tracks, so arrow keys, Page Up/Down, Home and
 * End all move it, and a screen reader hears a slider with its value. What is drawn is the
 * capsule and the line; the input itself is invisible.
 */
import { computed, useId } from 'vue'
import type { ClaudeUsageLimit } from '@/model/claude'
import { formatResetIn } from '@/common/format/countdown'

const props = withDefaults(
  defineProps<{
    modelValue: number
    /** The windows the ceiling is checked against, as the usage reading gives them. */
    windows: readonly ClaudeUsageLimit[]
    disabled?: boolean
    now?: number
  }>(),
  { disabled: false, now: () => Date.now() }
)

const emit = defineEmits<{ 'update:modelValue': [value: number] }>()

const MIN = 10
const MAX = 100
const SCALE_TICKS = [0, 25, 50, 75, 100] as const

const id = useId()

const clamped = computed(() => Math.min(MAX, Math.max(MIN, props.modelValue)))

// The input spans the same 0-100 scale the tracks are drawn on, so the line lands where the
// pointer is; values under the floor snap back to it.
function onInput(event: Event): void {
  const input = event.target as HTMLInputElement
  const value = Math.max(MIN, Number(input.value))
  if (Number(input.value) !== value) input.value = String(value)
  emit('update:modelValue', value)
}

function reached(limit: ClaudeUsageLimit): boolean {
  return limit.percent >= clamped.value
}

function resetLabel(limit: ClaudeUsageLimit): string {
  const left = formatResetIn(limit.resetsAt, props.now)
  return left ? `resets in ${left}` : ''
}
</script>

<template>
  <div
    class="ceiling-gauge relative select-none"
    :class="{ 'opacity-45': disabled }"
    :style="{ '--ceiling': `${clamped}%` }"
    data-testid="ceiling-gauge"
  >
    <ul class="flex flex-col gap-3 pt-7">
      <li v-for="limit in windows" :key="limit.key" class="flex flex-col gap-1">
        <div class="flex items-baseline justify-between gap-2 text-[11.5px]">
          <span class="text-text-muted">{{ limit.label }}</span>
          <span class="relative z-10 flex items-baseline gap-2 rounded bg-surface px-1">
            <span class="font-mono text-[10.5px] text-text-subtle">{{ resetLabel(limit) }}</span>
            <span
              class="font-mono tabular-nums"
              :class="reached(limit) && !disabled ? 'text-warn' : 'text-anchor'"
              >{{ Math.round(limit.percent) }}%</span
            >
          </span>
        </div>
        <div class="relative">
          <div class="h-1.5 overflow-hidden rounded-full bg-border">
            <span
              class="block h-full rounded-full transition-[width,background-color] duration-300"
              :class="reached(limit) && !disabled ? 'bg-warn' : 'bg-anchor'"
              :style="{ width: `${Math.min(100, Math.max(1.5, limit.percent))}%` }"
              :data-reached="reached(limit)"
            />
          </div>
          <span class="notch pointer-events-none absolute -top-1 h-3.5 w-0.5 rounded-full bg-accent" aria-hidden="true" />
        </div>
      </li>
      <li v-if="windows.length === 0" class="py-1 text-[11.5px] leading-relaxed text-text-subtle">
        No usage reading yet. The line still holds: the queue checks it before every task and step.
      </li>
    </ul>

    <!-- The waterline: from the capsule down through every track. -->
    <span class="waterline pointer-events-none absolute bottom-5 top-[18px] w-0" aria-hidden="true" />
    <span
      class="capsule pointer-events-none absolute top-0 grid h-[18px] min-w-9 place-items-center rounded-full bg-accent px-1.5 font-mono text-[10.5px] font-semibold tabular-nums text-accent-ink shadow-lift"
      aria-hidden="true"
      data-testid="ceiling-capsule"
    >
      {{ clamped }}%
    </span>

    <div class="mt-2 flex justify-between font-mono text-[10px] text-text-subtle" aria-hidden="true">
      <span v-for="tick in SCALE_TICKS" :key="tick">{{ tick }}</span>
    </div>

    <label :for="id" class="sr-only">Usage ceiling, in percent</label>
    <input
      :id="id"
      type="range"
      class="range absolute inset-x-0 top-0 bottom-5 w-full cursor-ew-resize opacity-0 disabled:cursor-not-allowed"
      :min="0"
      :max="MAX"
      step="1"
      :value="clamped"
      :disabled="disabled"
      :aria-valuetext="`${clamped} percent`"
      data-testid="ceiling-input"
      @input="onInput"
    />
  </div>
</template>

<style scoped>
/*
 * The line sits at the ceiling's share of the track width; the capsule centres on it but never
 * leaves the gauge, so 10% and 100% keep their labels on screen.
 */
.waterline {
  left: calc(var(--ceiling) - 0.5px);
  border-left: 1px dashed color-mix(in srgb, var(--color-accent) 35%, transparent);
}

.notch {
  left: calc(var(--ceiling) - 1px);
  box-shadow: 0 0 8px -1px color-mix(in srgb, var(--color-accent) 75%, transparent);
}

.capsule {
  left: clamp(0px, calc(var(--ceiling) - 18px), calc(100% - 36px));
}

/* A hairline thumb, so the value under the pointer is the one at the pointer, edge to edge. */
.range {
  appearance: none;
  background: transparent;
}

.range::-webkit-slider-thumb {
  width: 2px;
  height: 100%;
  appearance: none;
}

.range::-moz-range-thumb {
  width: 2px;
  border: 0;
}

/* The input is invisible, so keyboard focus is drawn on what it moves. */
.ceiling-gauge:has(.range:focus-visible) .capsule {
  outline: 2px solid var(--color-accent-strong);
  outline-offset: 2px;
}
</style>
