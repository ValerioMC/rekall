<script setup lang="ts">
/**
 * The shell's question before it quits or restarts while a Claude session is live, asked here so
 * it looks like every other dialog. The shell falls back to a system dialog when this is not
 * mounted (`rekallLeaveReady`).
 */
import { nextTick, onMounted, onUnmounted, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppDialog from '@/components/ui/AppDialog.vue'
import { answerLeave } from '@/common/native/desktop'

interface LeaveQuestion {
  readonly title: string
  readonly message: string
  readonly confirm: string
  readonly stay: string
}

const question = ref<LeaveQuestion | null>(null)
const stayButton = ref<InstanceType<typeof AppButton> | null>(null)

async function ask(event: Event): Promise<void> {
  question.value = (event as CustomEvent<LeaveQuestion>).detail
  await nextTick()
  ;(stayButton.value?.$el as HTMLElement | undefined)?.focus()
}

async function reply(confirmed: boolean): Promise<void> {
  if (question.value === null) return
  question.value = null
  await answerLeave(confirmed)
}

onMounted(() => {
  window.addEventListener('rekall:leave', ask)
  window.rekallLeaveReady = true
})

onUnmounted(() => {
  window.rekallLeaveReady = false
  window.removeEventListener('rekall:leave', ask)
})
</script>

<template>
  <AppDialog
    v-if="question"
    :label="question.title"
    role="alertdialog"
    panel-class="max-w-[440px]"
    panel-testid="leave-dialog"
    @dismiss="reply(false)"
  >
    <div class="p-5">
      <h2 class="mb-2 text-title font-semibold text-text">{{ question.title }}</h2>
      <p class="text-body leading-relaxed text-text-muted">{{ question.message }}</p>
    </div>
    <div class="flex justify-end gap-2 border-t border-border bg-canvas px-5 py-3">
      <AppButton ref="stayButton" variant="ghost" size="sm" data-testid="leave-stay" @click="reply(false)">
        {{ question.stay }}
      </AppButton>
      <AppButton variant="danger-quiet" size="sm" data-testid="leave-confirm" @click="reply(true)">
        {{ question.confirm }}
      </AppButton>
    </div>
  </AppDialog>
</template>
