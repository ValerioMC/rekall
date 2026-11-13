<script setup lang="ts">
import { nextTick, onBeforeUnmount, ref, watch } from 'vue'

export type PopoverMenuEntry =
  | Readonly<{ kind: 'heading'; label: string }>
  | Readonly<{ kind: 'separator' }>
  | Readonly<{ kind: 'item'; id: string; label: string; current?: boolean }>

/**
 * A small menu under its own trigger. Arrow keys walk the items, Escape closes and returns the
 * focus to the trigger, a press outside closes, and picking an item closes it too.
 */
defineProps<{ entries: readonly PopoverMenuEntry[]; label: string; align?: 'left' | 'right' }>()

const emit = defineEmits<{ pick: [id: string] }>()

const open = defineModel<boolean>('open', { default: false })
const host = ref<HTMLElement | null>(null)
const menu = ref<HTMLElement | null>(null)

function items(): HTMLElement[] {
  return Array.from(menu.value?.querySelectorAll<HTMLElement>('[role="menuitem"]') ?? [])
}

function close(returnFocus: boolean): void {
  open.value = false
  if (returnFocus) host.value?.querySelector<HTMLElement>('[data-popover-trigger]')?.focus()
}

function pick(id: string): void {
  emit('pick', id)
  close(true)
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    event.preventDefault()
    close(true)
    return
  }
  if (event.key !== 'ArrowDown' && event.key !== 'ArrowUp') return
  event.preventDefault()
  const all = items()
  const focused = all.findIndex((item) => item === document.activeElement)
  const step = event.key === 'ArrowDown' ? 1 : -1
  all[(focused + step + all.length) % all.length]?.focus()
}

function onPointerDown(event: PointerEvent): void {
  if (host.value && event.target instanceof Node && !host.value.contains(event.target)) close(false)
}

watch(open, async (isOpen) => {
  if (!isOpen) {
    window.removeEventListener('pointerdown', onPointerDown, true)
    return
  }
  window.addEventListener('pointerdown', onPointerDown, true)
  await nextTick()
  const all = items()
  all[Math.max(0, all.findIndex((item) => item.getAttribute('aria-current') === 'true'))]?.focus()
})

onBeforeUnmount(() => window.removeEventListener('pointerdown', onPointerDown, true))
</script>

<template>
  <div ref="host" class="relative inline-flex" data-testid="popover-menu">
    <slot name="trigger" :open="open" :toggle="() => (open = !open)" />
    <Transition name="popover">
      <div
        v-if="open"
        ref="menu"
        class="absolute top-full z-(--z-overlay) mt-1.5 flex min-w-[220px] flex-col gap-0.5 rounded-card border border-border-strong bg-surface p-1.5 shadow-(--shadow-lift)"
        :class="align === 'right' ? 'right-0 origin-top-right' : 'left-0 origin-top-left'"
        role="menu"
        :aria-label="label"
        data-testid="popover-menu-list"
        @keydown="onKeydown"
      >
        <template v-for="(entry, index) in entries" :key="index">
          <p v-if="entry.kind === 'heading'" class="px-2.5 pb-0.5 pt-1.5 text-caption font-medium text-text-subtle" role="presentation">
            {{ entry.label }}
          </p>
          <hr v-else-if="entry.kind === 'separator'" class="my-1 border-border" role="separator" />
          <button
            v-else
            type="button"
            role="menuitem"
            class="focus-ring flex w-full cursor-pointer items-center gap-2.5 rounded-control px-2.5 py-2 text-left text-small transition-colors duration-(--motion-quick)"
            :class="entry.current ? 'bg-surface-raised text-text' : 'text-text-muted hover:bg-surface-raised hover:text-text'"
            :aria-current="entry.current ? 'true' : undefined"
            data-testid="popover-menu-item"
            @click="pick(entry.id)"
          >
            {{ entry.label }}
          </button>
        </template>
      </div>
    </Transition>
  </div>
</template>
