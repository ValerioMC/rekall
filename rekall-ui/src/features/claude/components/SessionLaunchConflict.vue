<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import AppDialog from '@/components/ui/AppDialog.vue'
import { useOpenSession } from '@/features/claude/composables/useOpenSession'
import { SESSION_PROVIDER_LABEL } from '@/features/claude/model/session-provider'
import { useTerminalStore } from '@/features/claude/store/terminal.store'

const terminals = useTerminalStore()
const openSession = useOpenSession()
const cancelFallback = ref<InstanceType<typeof AppButton> | null>(null)
watch(cancelFallback, (button) => (button?.$el as HTMLElement | undefined)?.focus(), { flush: 'post' })

const existingLabel = computed(() => {
  const session = terminals.launchConflict?.session
  if (!session) return ''
  const view = session.kind === 'CHAT' ? 'Chat' : 'Terminal'
  return `${SESSION_PROVIDER_LABEL[session.provider]} ${view}`
})

function openExisting(): void {
  const session = terminals.launchConflict?.session
  if (!session) return
  terminals.resolveLaunchConflict(false)
  openSession(session)
}
</script>

<template>
  <AppConfirm
    v-if="terminals.launchConflict"
    title="A session is already open"
    :body="terminals.launchConflict.detail"
    :blast="`Retrying will end the open ${existingLabel} session.`"
    :confirm-label="`Close and retry with ${SESSION_PROVIDER_LABEL[terminals.launchConflict.provider]}`"
    @cancel="terminals.resolveLaunchConflict(false)"
    @confirm="terminals.resolveLaunchConflict(true)"
  >
    <AppButton class="mt-3" size="sm" variant="quiet" @click="openExisting">
      Open {{ existingLabel }}
    </AppButton>
  </AppConfirm>
  <AppDialog
    v-if="terminals.chatFallback"
    label="Codex Chat is unavailable"
    panel-class="max-w-[440px]"
    @dismiss="terminals.resolveChatFallback(false)"
  >
    <div class="p-5">
      <h2 class="mb-2 text-title font-semibold text-text">Codex Chat is unavailable</h2>
      <p class="text-body leading-relaxed text-text-muted">{{ terminals.chatFallback.detail }}</p>
    </div>
    <div class="flex justify-end gap-2 border-t border-border bg-canvas px-5 py-3">
      <AppButton ref="cancelFallback" variant="ghost" size="sm" @click="terminals.resolveChatFallback(false)">Cancel</AppButton>
      <AppButton variant="primary" size="sm" @click="terminals.resolveChatFallback(true)">Open in Terminal</AppButton>
    </div>
  </AppDialog>
</template>
