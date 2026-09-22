<script setup lang="ts">
import { ref } from 'vue'
import AppCheckbox from '@/components/ui/AppCheckbox.vue'
import {
  canNotify,
  claimNotificationsEnabled,
  requestNotificationPermission,
  setClaimNotificationsEnabled
} from '@/common/native/notify'

const enabled = ref(claimNotificationsEnabled())
const refused = ref(false)
const available = canNotify()

// The browser grants permission only from a click, which is why it is asked here and nowhere else.
async function toggle(value: boolean): Promise<void> {
  enabled.value = value
  setClaimNotificationsEnabled(value)
  refused.value = value ? !(await requestNotificationPermission()) : false
}
</script>

<template>
  <section aria-labelledby="notifications-heading">
    <h3 id="notifications-heading" class="eyebrow mb-2">Notifications</h3>
    <AppCheckbox
      :model-value="enabled"
      label="Tell me when a session claims work while Rekall is in the background"
      described-by="notifications-hint"
      data-testid="notify-claims"
      @update:model-value="toggle"
    />
    <p id="notifications-hint" class="mt-1.5 text-[11.5px] leading-relaxed text-text-subtle">
      <template v-if="!available">This window cannot post notifications.</template>
      <template v-else-if="refused">
        The system refused. Allow notifications for Rekall in its settings, then turn this on again.
      </template>
      <template v-else>
        A claimed step, or a stepless task whose wrapup claimed it, joins the Review list (the checklist icon in the top
        bar) either way. Stored on this machine, not in the database.
      </template>
    </p>
  </section>
</template>
