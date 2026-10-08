<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { useModalGate } from '@/composables/useModalGate'
import { trapTabKey } from '@/common/a11y/focus-trap'
import { enterDialog, isTopDialog, leaveDialog } from '@/common/a11y/dialog-stack'

/*
 * The one modal shell: backdrop, panel, Escape, focus trap and the modal gate. What goes inside,
 * which element takes focus first and what dismissing means stay with the dialog that uses it.
 */
const props = withDefaults(
  defineProps<{
    label: string
    role?: 'dialog' | 'alertdialog'
    /** False while something must finish first, or a control inside owns Escape: the dialog then lets it through. */
    dismissible?: boolean
    /** False lets Tab leave the panel, for a dialog that has handed focus to the system. */
    trapped?: boolean
    placement?: 'center' | 'right'
    panelClass?: string
    panelTestid?: string
  }>(),
  { role: 'dialog', dismissible: true, trapped: true, placement: 'center', panelClass: '', panelTestid: undefined }
)

const emit = defineEmits<{ dismiss: [reason: 'escape' | 'backdrop'] }>()

const panel = ref<HTMLElement | null>(null)
const { open: openModal, close: closeModal } = useModalGate()
let token: symbol | null = null

function onKeydown(event: KeyboardEvent): void {
  if (token === null || !isTopDialog(token)) return
  if (event.key === 'Escape') {
    if (!props.dismissible) return
    event.stopPropagation()
    emit('dismiss', 'escape')
    return
  }
  if (panel.value && props.trapped) trapTabKey(panel.value, event)
}

function onBackdrop(): void {
  if (props.dismissible) emit('dismiss', 'backdrop')
}

onMounted(() => {
  token = enterDialog()
  openModal()
  window.addEventListener('keydown', onKeydown, true)
})

onUnmounted(() => {
  if (token !== null) leaveDialog(token)
  closeModal()
  window.removeEventListener('keydown', onKeydown, true)
})

defineExpose({ panel })
</script>

<template>
  <div
    class="fixed inset-0 z-(--z-modal) bg-black/65 backdrop-blur-sm"
    :class="placement === 'center' && 'grid place-items-center p-5'"
    @click.self="onBackdrop"
  >
    <div
      ref="panel"
      :class="[
        placement === 'center'
          ? 'dialog-panel w-full overflow-hidden rounded-card border border-border-strong bg-surface shadow-modal'
          : 'slide-over-panel absolute inset-y-0 right-0 flex w-full flex-col border-l border-border-strong bg-surface shadow-modal',
        panelClass
      ]"
      :role="role"
      aria-modal="true"
      :aria-label="label"
      :data-testid="panelTestid"
    >
      <slot />
    </div>
    <slot name="overlay" />
  </div>
</template>
