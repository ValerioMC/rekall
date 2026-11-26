<script setup lang="ts">
import { ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import { SESSION_PROVIDER_LABEL } from '@/features/claude/model/session-provider'
import type { Terminal } from '@/features/claude/model/terminal'

/** Ends a chat's whole session, only once the person has confirmed it. */
defineProps<{ session: Terminal }>()

const emit = defineEmits<{ stop: [] }>()

const confirming = ref(false)

function confirmStop(): void {
  confirming.value = false
  emit('stop')
}
</script>

<template>
  <AppButton variant="danger" size="sm" data-testid="chat-stop-session" @click="confirming = true">
    Stop session
  </AppButton>
  <AppConfirm
    v-if="confirming"
    title="Stop this session?"
    :body="`${SESSION_PROVIDER_LABEL[session.provider]} stops where it is. The conversation stays in the provider’s history.`"
    :blast="session.anchors"
    confirm-label="Stop session"
    @cancel="confirming = false"
    @confirm="confirmStop"
  />
</template>
