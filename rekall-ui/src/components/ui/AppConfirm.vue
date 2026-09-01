<script setup lang="ts">
import { nextTick, onMounted, onUnmounted, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import { useModalGate } from '@/composables/useModalGate'
import { trapTabKey } from '@/common/a11y/focus-trap'

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

const panel = ref<HTMLElement | null>(null)
// The safe action, focused by default: a stray Enter should keep the record, not destroy it.
const cancelButton = ref<InstanceType<typeof AppButton> | null>(null)

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    event.stopPropagation()
    emit('cancel')
    return
  }
  if (panel.value) trapTabKey(panel.value, event)
}

onMounted(async () => {
  openModal()
  window.addEventListener('keydown', onKeydown, true)
  await nextTick()
  ;(cancelButton.value?.$el as HTMLElement | undefined)?.focus()
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
      ref="panel"
      class="w-full max-w-[440px] overflow-hidden rounded-[var(--radius-card)] border border-border-strong bg-surface shadow-lift"
      role="alertdialog"
      aria-modal="true"
      :aria-label="title"
    >
      <div class="p-5">
        <h2 class="mb-2 text-[16px] font-semibold text-text">{{ title }}</h2>
        <p class="text-[13px] leading-relaxed text-text-muted">{{ body }}</p>
        <p
          class="relative mt-3.5 overflow-hidden rounded-[var(--radius-control)] border border-danger bg-danger-soft py-2.5 pl-4 pr-3 font-mono text-[12px] text-danger"
        >
          <span
            class="absolute inset-y-0 left-0 w-2.5"
            style="background-image: repeating-linear-gradient(135deg, var(--color-danger) 0 3px, transparent 3px 7px); opacity: 0.55"
            aria-hidden="true"
          />
          {{ blast }}
        </p>
      </div>
      <div class="flex justify-end gap-2 border-t border-border bg-canvas px-5 py-3">
        <AppButton ref="cancelButton" variant="ghost" size="sm" @click="emit('cancel')">
          Keep it
        </AppButton>
        <AppButton variant="danger" size="sm" @click="emit('confirm')">
          {{ confirmLabel }}
        </AppButton>
      </div>
    </div>
  </div>
</template>
