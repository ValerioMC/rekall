<script setup lang="ts">
import { ref } from 'vue'
import { useAnchoredMenu } from '@/composables/useAnchoredMenu'

/**
 * The narrow key joined to the right of Run here. Its menu offers the ways to run beyond starting
 * a terminal now; it takes the main key's look through `ready` and `pressed`, and no press while `busy`.
 */
defineProps<{ ready: boolean; pressed: boolean; busy: boolean }>()

const emit = defineEmits<{ runInQueue: [] }>()

const MENU_WIDTH = 280

const arrow = ref<HTMLButtonElement | null>(null)

const { open, menu, placement, toggle, close, handleKeydown } = useAnchoredMenu(() => arrow.value, MENU_WIDTH)

function runInQueue(): void {
  close(true)
  emit('runInQueue')
}
</script>

<template>
  <button
    ref="arrow"
    type="button"
    class="focus-ring relative inline-flex h-7 shrink-0 items-center justify-center rounded-key rounded-l-none px-1.5"
    :class="[
      ready
        ? pressed
          ? 'key-lit'
          : 'key-gold font-semibold'
        : 'bg-transparent text-text-subtle transition-colors hover:bg-surface-raised hover:text-text-muted',
      busy && 'cursor-default'
    ]"
    aria-haspopup="menu"
    :aria-expanded="open"
    :aria-disabled="busy"
    aria-label="More ways to run"
    title="More ways to run"
    data-testid="run-menu-arrow"
    @click="busy ? undefined : toggle()"
  >
    <svg
      class="size-3.5 transition-transform"
      :class="open ? 'rotate-180' : ''"
      viewBox="0 0 16 16"
      fill="none"
      aria-hidden="true"
    >
      <path
        d="m4 6 4 4 4-4"
        stroke="currentColor"
        stroke-width="1.6"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  </button>

  <Teleport to="body">
    <div
      v-if="open"
      ref="menu"
      role="menu"
      aria-label="More ways to run"
      class="fixed z-[var(--z-popover)] rounded-card border border-border-strong bg-surface-raised p-1 shadow-lift"
      :style="placement"
      data-testid="run-menu"
      @keydown="handleKeydown"
    >
      <button
        type="button"
        role="menuitem"
        class="focus-ring flex w-full cursor-pointer flex-col items-start gap-0.5 rounded-control px-2.5 py-2 text-left transition-colors hover:bg-surface"
        data-testid="run-in-queue"
        @click="runInQueue"
      >
        <span class="text-small font-medium text-text">Run in queue</span>
        <span class="text-caption leading-snug text-text-muted"
          >Runs the whole task after the ones already queued</span
        >
      </button>
    </div>
  </Teleport>
</template>
