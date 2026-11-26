<script setup lang="ts">
import { computed } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import { useOpenSession } from '@/features/claude/composables/useOpenSession'
import { SESSION_PROVIDER_LABEL } from '@/features/claude/model/session-provider'
import { useTerminalStore } from '@/features/claude/store/terminal.store'

const terminals = useTerminalStore()
const openSession = useOpenSession()
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
</template>
