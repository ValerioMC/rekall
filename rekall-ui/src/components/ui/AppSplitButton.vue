<script setup lang="ts" generic="Id extends string">
import { nextTick, onUnmounted, ref, type ComponentPublicInstance } from 'vue'
import AppButton from './AppButton.vue'

/** One of the other actions a split button's arrow offers beside its main one. */
export interface SplitButtonOption<OptionId extends string> {
  readonly id: OptionId
  readonly label: string
  readonly hint: string
}

/**
 * A main action with its alternatives behind an arrow on the same key, so a row of related
 * actions takes one button. Without options it is a plain button.
 */
withDefaults(
  defineProps<{
    options: readonly SplitButtonOption<Id>[]
    /** What the arrow opens, for assistive tech: "More ways to fix". */
    menuLabel: string
    variant?: 'primary' | 'secondary'
    size?: 'sm' | 'md'
    loading?: boolean
    disabled?: boolean
  }>(),
  { variant: 'secondary', size: 'md', loading: false, disabled: false }
)

const emit = defineEmits<{ click: []; choose: [id: Id] }>()

defineOptions({ inheritAttrs: false })

const MENU_WIDTH = 280
const EDGE = 8

const open = ref(false)
const arrow = ref<ComponentPublicInstance | null>(null)
const menu = ref<HTMLElement | null>(null)
const placement = ref<Record<string, string>>({})

function arrowElement(): HTMLElement | null {
  const element: unknown = arrow.value?.$el
  return element instanceof HTMLElement ? element : null
}

function place(): void {
  const box = arrowElement()?.getBoundingClientRect()
  if (!box) return
  const left = Math.max(EDGE, Math.min(box.right - MENU_WIDTH, window.innerWidth - MENU_WIDTH - EDGE))
  placement.value = { left: `${left}px`, top: `${box.bottom + 4}px`, width: `${MENU_WIDTH}px` }
}

function menuItems(): HTMLElement[] {
  return [...(menu.value?.querySelectorAll<HTMLElement>('[role="menuitem"]') ?? [])]
}

function close(returnFocus: boolean): void {
  open.value = false
  document.removeEventListener('pointerdown', closeOnOutside, true)
  if (returnFocus) arrowElement()?.focus()
}

function closeOnOutside(event: Event): void {
  if (!(event.target instanceof Node)) return close(false)
  if (arrowElement()?.contains(event.target) || menu.value?.contains(event.target)) return
  close(false)
}

async function toggle(): Promise<void> {
  if (open.value) return close(true)
  place()
  open.value = true
  document.addEventListener('pointerdown', closeOnOutside, true)
  await nextTick()
  menuItems()[0]?.focus()
}

function choose(id: Id): void {
  close(true)
  emit('choose', id)
}

function moveFocus(event: KeyboardEvent): void {
  if (event.key === 'Escape' || event.key === 'Tab') {
    event.stopPropagation()
    return close(event.key === 'Escape')
  }
  const step = event.key === 'ArrowDown' ? 1 : event.key === 'ArrowUp' ? -1 : 0
  if (step === 0) return
  event.preventDefault()
  const items = menuItems()
  const at = items.indexOf(document.activeElement as HTMLElement)
  items[(at + step + items.length) % items.length]?.focus()
}

onUnmounted(() => document.removeEventListener('pointerdown', closeOnOutside, true))
</script>

<template>
  <div class="inline-flex items-stretch gap-px" data-testid="split-button">
    <AppButton
      v-bind="$attrs"
      :variant="variant"
      :size="size"
      :loading="loading"
      :disabled="disabled"
      :class="options.length ? 'rounded-r-none' : ''"
      @click="emit('click')"
    >
      <slot />
    </AppButton>
    <AppButton
      v-if="options.length"
      ref="arrow"
      :variant="variant"
      :size="size"
      :disabled="disabled || loading"
      class="rounded-l-none !px-1.5"
      aria-haspopup="menu"
      :aria-expanded="open"
      :aria-label="menuLabel"
      :title="menuLabel"
      data-testid="split-button-arrow"
      @click="toggle"
    >
      <svg class="size-3.5 transition-transform" :class="open ? 'rotate-180' : ''" viewBox="0 0 16 16" fill="none" aria-hidden="true">
        <path d="m4 6 4 4 4-4" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </AppButton>
  </div>

  <Teleport to="body">
    <div
      v-if="open"
      ref="menu"
      role="menu"
      :aria-label="menuLabel"
      class="fixed z-[var(--z-popover)] rounded-card border border-border-strong bg-surface-raised p-1 shadow-lift"
      :style="placement"
      data-testid="split-button-menu"
      @keydown="moveFocus"
    >
      <button
        v-for="option in options"
        :key="option.id"
        type="button"
        role="menuitem"
        class="focus-ring flex w-full cursor-pointer flex-col items-start gap-0.5 rounded-control px-2.5 py-2 text-left transition-colors hover:bg-surface"
        :data-testid="`split-button-option-${option.id}`"
        @click="choose(option.id)"
      >
        <span class="text-small font-medium text-text">{{ option.label }}</span>
        <span class="text-caption leading-snug text-text-muted">{{ option.hint }}</span>
      </button>
    </div>
  </Teleport>
</template>
