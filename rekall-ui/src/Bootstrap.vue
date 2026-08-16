<script setup lang="ts">
import { onMounted, ref } from 'vue'
import App from '@/App.vue'
import FirstRunSetup from '@/components/setup/FirstRunSetup.vue'
import DatabaseUnreachable from '@/components/setup/DatabaseUnreachable.vue'
import AppLogo from '@/components/ui/AppLogo.vue'
import { fetchDatabaseStatus } from '@/api/settings.api'
import type { DatabaseStatus } from '@/model/settings'

/**
 * Decides, once, which of three things the console mounts as: the working application, the
 * first-run wizard, or the recovery screen for a database that has stopped being reachable.
 *
 * A router would be the conventional place for this; the console does not have one (see
 * `main.ts`), and this is a boot-time branch rather than navigation, so a plain conditional
 * render is what the rest of this codebase would do here too.
 */
const status = ref<DatabaseStatus | null>(null)
const failed = ref(false)

onMounted(async () => {
  try {
    status.value = await fetchDatabaseStatus()
  } catch {
    failed.value = true
  }
})
</script>

<template>
  <template v-if="status">
    <App v-if="status.status === 'READY'" />
    <FirstRunSetup v-else-if="status.status === 'SETUP_NEEDED'" />
    <DatabaseUnreachable v-else :status="status" />
  </template>
  <div
    v-else-if="failed"
    class="grid h-full place-items-center p-6 text-center text-[13px] text-text-subtle"
  >
    Rekall couldn't be reached. Make sure it's running, then reload the page.
  </div>
  <!-- The one instant nothing is known yet: which of the three screens above this becomes.
       A blank canvas reads as broken; the mark fading in reads as a product about to answer. -->
  <div v-else class="fade-in grid h-full place-items-center" role="status" aria-live="polite">
    <span class="sr-only">Loading</span>
    <AppLogo :size="40" class="halo animate-pulse rounded-[9px]" aria-hidden="true" />
  </div>
</template>
