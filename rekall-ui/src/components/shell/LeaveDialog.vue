<script setup lang="ts">
/**
 * The shell's question before it quits or restarts while a Claude session is live, asked here so
 * it looks like every other dialog. The shell falls back to a system dialog when this is not
 * mounted (`rekallLeaveReady`).
 */
import { nextTick, onMounted, onUnmounted, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import { useModalGate } from '@/composables/useModalGate'
import { trapTabKey } from '@/common/a11y/focus-trap'
import { answerLeave } from '@/common/native/desktop'

interface LeaveQuestion {
  readonly title: string
  readonly message: string
  readonly confirm: string
  readonly stay: string
}

const { open: openModal, close: closeModal } = useModalGate()

const question = ref<LeaveQuestion | null>(null)
const panel = ref<HTMLElement | null>(null)
const stayButton = ref<InstanceType<typeof AppButton> | null>(null)

async function ask(event: Event): Promise<void> {
  if (question.value === null) openModal()
  question.value = (event as CustomEvent<LeaveQuestion>).detail
  await nextTick()
  ;(stayButton.value?.$el as HTMLElement | undefined)?.focus()
}

async function reply(confirmed: boolean): Promise<void> {
  if (question.value === null) return
  question.value = null
  closeModal()
  await answerLeave(confirmed)
}

function onKeydown(event: KeyboardEvent): void {
  if (question.value === null) return
  if (event.key === 'Escape') {
    event.stopPropagation()
    void reply(false)
    return
  }
  if (panel.value) trapTabKey(panel.value, event)
}

onMounted(() => {
  window.addEventListener('rekall:leave', ask)
  window.addEventListener('keydown', onKeydown, true)
  window.rekallLeaveReady = true
})

onUnmounted(() => {
  window.rekallLeaveReady = false
  window.removeEventListener('rekall:leave', ask)
  window.removeEventListener('keydown', onKeydown, true)
  if (question.value !== null) closeModal()
})
</script>

<template>
  <div
    v-if="question"
    class="fixed inset-0 z-(--z-modal) grid place-items-center bg-black/60 p-5 backdrop-blur-sm"
    @click.self="reply(false)"
  >
    <div
      ref="panel"
      class="dialog-panel w-full max-w-[440px] overflow-hidden rounded-[var(--radius-card)] border border-border-strong bg-surface shadow-lift"
      role="alertdialog"
      aria-modal="true"
      :aria-label="question.title"
      data-testid="leave-dialog"
    >
      <div class="p-5">
        <h2 class="mb-2 text-[16px] font-semibold text-text">{{ question.title }}</h2>
        <p class="text-[13px] leading-relaxed text-text-muted">{{ question.message }}</p>
      </div>
      <div class="flex justify-end gap-2 border-t border-border bg-canvas px-5 py-3">
        <AppButton ref="stayButton" variant="ghost" size="sm" data-testid="leave-stay" @click="reply(false)">
          {{ question.stay }}
        </AppButton>
        <AppButton variant="danger-quiet" size="sm" data-testid="leave-confirm" @click="reply(true)">
          {{ question.confirm }}
        </AppButton>
      </div>
    </div>
  </div>
</template>
