<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { useNavigationHistory } from '@/composables/useNavigationHistory'
import { useModalGate } from '@/composables/useModalGate'
import {
  historyDirectionOf,
  historyShortcutLabel,
  runsOnMac,
  type HistoryDirection
} from '@/common/shell/history-shortcut'

/**
 * Back and forward through the screens the reader has visited, as one rocker beside the
 * wordmark. A side with nowhere to go dims and refuses the press; a press nudges its chevron
 * the way it travels.
 */
const { canGoBack, canGoForward, goBack, goForward } = useNavigationHistory()
const { isModalOpen } = useModalGate()
const nudged = ref<HistoryDirection | null>(null)
const mac = runsOnMac(navigator.platform)

function travel(direction: HistoryDirection): void {
  nudged.value = null
  requestAnimationFrame(() => (nudged.value = direction))
  if (direction === 'back') goBack()
  else goForward()
}

// Also fires while typing: the chord has no text-editing meaning. A modal owns the keyboard.
function onKeydown(event: KeyboardEvent): void {
  const direction = historyDirectionOf(event, mac)
  if (!direction || isModalOpen.value > 0) return
  event.preventDefault()
  travel(direction)
}

onMounted(() => window.addEventListener('keydown', onKeydown))
onUnmounted(() => window.removeEventListener('keydown', onKeydown))
</script>

<template>
  <div class="rocker flex shrink-0 items-center rounded-control bg-canvas p-0.5" role="group" aria-label="History">
    <button
      type="button"
      class="rocker-key focus-ring"
      :disabled="!canGoBack"
      :title="`Back (${historyShortcutLabel('back', mac)})`"
      aria-label="Back"
      data-testid="history-back"
      @click="travel('back')"
    >
      <svg
        class="size-3"
        :class="nudged === 'back' && 'nudge-back'"
        viewBox="0 0 12 12"
        fill="none"
        aria-hidden="true"
        @animationend="nudged = null"
      >
        <path d="M7.4 2.6 4 6l3.4 3.4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </button>
    <span class="h-3 w-px bg-border" aria-hidden="true" />
    <button
      type="button"
      class="rocker-key focus-ring"
      :disabled="!canGoForward"
      :title="`Forward (${historyShortcutLabel('forward', mac)})`"
      aria-label="Forward"
      data-testid="history-forward"
      @click="travel('forward')"
    >
      <svg
        class="size-3"
        :class="nudged === 'forward' && 'nudge-forward'"
        viewBox="0 0 12 12"
        fill="none"
        aria-hidden="true"
        @animationend="nudged = null"
      >
        <path d="M4.6 2.6 8 6 4.6 9.4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </button>
  </div>
</template>

<style scoped>
.rocker-key {
  display: grid;
  place-items: center;
  width: 22px;
  height: 22px;
  border-radius: 5px;
  color: var(--color-text-muted);
  transition:
    background-color var(--motion-quick) ease,
    color var(--motion-quick) ease;
}

.rocker-key:hover:not(:disabled) {
  background: var(--color-surface-raised);
  color: var(--color-text);
}

.rocker-key:active:not(:disabled) {
  background: color-mix(in srgb, var(--color-surface-raised) 70%, var(--color-canvas));
}

.rocker-key:disabled {
  color: var(--color-text-subtle);
  opacity: 0.35;
  cursor: default;
}

.nudge-back {
  animation: rocker-nudge-back 220ms cubic-bezier(0.3, 0.7, 0.2, 1);
}

.nudge-forward {
  animation: rocker-nudge-forward 220ms cubic-bezier(0.3, 0.7, 0.2, 1);
}

@keyframes rocker-nudge-back {
  40% {
    transform: translateX(-2.5px);
  }
}

@keyframes rocker-nudge-forward {
  40% {
    transform: translateX(2.5px);
  }
}

@media (prefers-reduced-motion: reduce) {
  .nudge-back,
  .nudge-forward {
    animation: none;
  }
}
</style>
