<script setup lang="ts">
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import AppDialog from '@/components/ui/AppDialog.vue'
import { useSendBackGuardStore } from '@/features/review/store/sendBackGuard.store'

const guard = useSendBackGuardStore()
const { pending } = storeToRefs(guard)
</script>

<template>
  <AppDialog v-if="pending" label="Send back" role="alertdialog" panel-class="max-w-[440px]" @dismiss="guard.cancel()">
    <div class="p-5" data-testid="send-back-confirm">
      <h2 class="mb-2 text-title font-semibold text-text">Send {{ pending.subject }} back?</h2>
      <p class="text-body leading-relaxed text-text-muted">It returns to the session as open work. Only a session can claim it again.</p>
    </div>
    <div class="flex justify-end gap-2 border-t border-border bg-canvas px-5 py-3">
      <AppButton variant="ghost" size="sm" data-testid="send-back-cancel" @click="guard.cancel()">Keep it claimed</AppButton>
      <AppButton variant="primary" size="sm" data-testid="send-back-confirm-yes" @click="guard.confirm()">Send back</AppButton>
    </div>
  </AppDialog>
</template>
