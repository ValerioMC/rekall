<script setup lang="ts">
/**
 * The place between two sessions, or after the last, where something being dragged runs in a
 * session of its own. It exists only while something is held; under the pointer it says what the
 * drop does.
 */
defineProps<{
  active: boolean
  /** What the held thing is: a step or draft runs alone, a session keeps its steps together. */
  label: string
  detail: string
  idleLabel: string
}>()

const emit = defineEmits<{ hover: []; drop: [] }>()

function hoverWhileDragging(event: DragEvent): void {
  event.preventDefault()
  emit('hover')
}

function dropHere(event: DragEvent): void {
  event.preventDefault()
  emit('drop')
}
</script>

<template>
  <div
    class="mx-2 my-1 flex items-center gap-2.5 rounded-control border px-3 text-caption font-medium transition-[min-height,background-color] duration-(--motion-quick)"
    :class="
      active
        ? 'min-h-14 border-session bg-session/15 text-session-strong'
        : 'min-h-6.5 border-dashed border-border-strong text-text-subtle'
    "
    data-testid="plan-session-slot"
    :data-active="active"
    @dragenter="hoverWhileDragging"
    @dragover="hoverWhileDragging"
    @drop="dropHere"
  >
    <svg class="size-2.5 shrink-0" viewBox="0 0 12 12" fill="none" aria-hidden="true">
      <path d="M6 2.5v7M2.5 6h7" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
    </svg>
    <span v-if="active" class="flex min-w-0 flex-col gap-px">
      <span class="text-body">{{ label }}</span>
      <span class="font-normal text-text-muted">{{ detail }}</span>
    </span>
    <span v-else>{{ idleLabel }}</span>
  </div>
</template>
