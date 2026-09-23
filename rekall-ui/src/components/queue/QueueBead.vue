<script setup lang="ts">
/**
 * One queued task as a bead on the relay: the same mark in the top-bar strand and beside its row
 * in the queue panel, so a glance at either reads the same way.
 *
 *   QUEUED    a hairline ring: waiting its turn.
 *   RUNNING   an amber bead with a slow pulse, only while the queue is actually running it.
 *   FINISHED  filled green: its work is claimed or accepted.
 *   SKIPPED   filled in the archive's verdigris: there was nothing on it to run.
 *   FAILED    a red ring with a bar through it: its session could not open, or ended early.
 *
 * Settling (to FINISHED, SKIPPED or FAILED) lands once as a small pop; a list loading in does not.
 */
import { onUnmounted, ref, watch } from 'vue'
import type { RunQueueItemState } from '@/model/runQueue'

const props = withDefaults(
  defineProps<{
    state: RunQueueItemState
    /** Whether a RUNNING bead may pulse: false while the queue is holding or stopped. */
    live?: boolean
    size?: 'sm' | 'md'
  }>(),
  { live: true, size: 'sm' }
)

const STATE_CLASS: Readonly<Record<RunQueueItemState, string>> = {
  QUEUED: 'border border-border-strong bg-transparent',
  RUNNING: 'bg-accent',
  FINISHED: 'bg-safe',
  SKIPPED: 'bg-filed',
  FAILED: 'border border-danger bg-danger-soft'
}

const popping = ref(false)
let popTimer: ReturnType<typeof setTimeout> | undefined

watch(
  () => props.state,
  (next, previous) => {
    if (next === previous || next === 'QUEUED' || next === 'RUNNING') return
    popping.value = false
    clearTimeout(popTimer)
    requestAnimationFrame(() => {
      popping.value = true
      popTimer = setTimeout(() => (popping.value = false), 340)
    })
  }
)

onUnmounted(() => clearTimeout(popTimer))
</script>

<template>
  <span
    class="relative inline-grid shrink-0 place-items-center rounded-full"
    :class="[
      size === 'sm' ? 'size-[7px]' : 'size-[11px]',
      STATE_CLASS[state],
      { 'bead-live': state === 'RUNNING' && live, settle: popping }
    ]"
    :data-state="state"
    data-testid="queue-bead"
    aria-hidden="true"
  >
    <span v-if="state === 'FAILED'" class="block h-px w-[60%] rotate-[-45deg] bg-danger" />
  </span>
</template>

<style scoped>
.bead-live {
  animation: step-pulse 1.7s ease-out infinite;
}
</style>
