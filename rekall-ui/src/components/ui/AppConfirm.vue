<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import { useModalGate } from '@/composables/useModalGate'

/**
 * A confirmation that states its blast radius.
 *
 * The screens this replaces deleted a project, its tasks and every note under it on one
 * unguarded click. Naming what goes with it is the whole point: "are you sure" asks a question
 * the person cannot answer without the number.
 */
defineProps<{
  title: string
  body: string
  /** What will be destroyed, in the interface's own terms. */
  blast: string
  confirmLabel: string
}>()

const emit = defineEmits<{ cancel: []; confirm: [] }>()

/**
 * The console's single-key shortcuts are inert while this is open.
 *
 * This dialog has no field to hold focus, so nothing else stopped a key from reaching the
 * console underneath: pressing `W` over "Delete wrapup?" switched the pane behind the dialog
 * and took the dialog with it. A question about destroying something has to be answered before
 * anything else happens.
 */
const { open: openModal, close: closeModal } = useModalGate()

const confirmButton = ref<InstanceType<typeof AppButton> | null>(null)

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    event.stopPropagation()
    emit('cancel')
  }
}

onMounted(() => {
  openModal()
  window.addEventListener('keydown', onKeydown, true)
})

onUnmounted(() => {
  closeModal()
  window.removeEventListener('keydown', onKeydown, true)
})
</script>

<template>
  <div
    class="fixed inset-0 z-(--z-modal) grid place-items-center bg-black/60 p-5 backdrop-blur-sm"
    @click.self="emit('cancel')"
  >
    <div
      class="w-full max-w-[440px] overflow-hidden rounded-[var(--radius-card)] border border-border-strong bg-surface shadow-lift"
      role="alertdialog"
      aria-modal="true"
      :aria-label="title"
    >
      <div class="p-5">
        <h2 class="mb-2 text-[16px] font-semibold text-text">{{ title }}</h2>
        <p class="text-[13px] leading-relaxed text-text-muted">{{ body }}</p>
        <p
          class="mt-3.5 rounded-[var(--radius-control)] border border-danger bg-danger-soft px-3 py-2.5 font-mono text-[12px] text-danger"
        >
          {{ blast }}
        </p>
      </div>
      <div class="flex justify-end gap-2 border-t border-border bg-canvas px-5 py-3">
        <AppButton variant="ghost" size="sm" @click="emit('cancel')">Keep it</AppButton>
        <AppButton ref="confirmButton" variant="danger" size="sm" @click="emit('confirm')">
          {{ confirmLabel }}
        </AppButton>
      </div>
    </div>
  </div>
</template>
