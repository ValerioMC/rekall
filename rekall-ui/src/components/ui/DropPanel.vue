<script setup lang="ts">
/**
 * A panel under its own trigger that stays open while its controls are used, unlike a menu that
 * closes on a pick. Escape and a press outside close it.
 */
defineProps<{ label: string }>()

const open = defineModel<boolean>('open', { default: false })

function onKeydown(event: KeyboardEvent): void {
  if (event.key !== 'Escape' || !open.value) return
  event.stopPropagation()
  open.value = false
}
</script>

<template>
  <div class="relative" data-testid="drop-panel" @keydown="onKeydown">
    <slot name="trigger" :open="open" :toggle="() => (open = !open)" />

    <div
      v-if="open"
      class="rise absolute right-0 top-[calc(100%+6px)] z-(--z-overlay) rounded-card border border-border-strong bg-surface-raised shadow-(--shadow-lift)"
      role="group"
      :aria-label="label"
      data-testid="drop-panel-body"
    >
      <slot />
    </div>

    <button v-if="open" type="button" class="fixed inset-0 z-(--z-sticky) cursor-default" tabindex="-1" aria-hidden="true" @click="open = false" />
  </div>
</template>
