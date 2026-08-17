<script setup lang="ts">
import { onMounted, ref } from 'vue'
import FirstRunSetup from '@/components/setup/FirstRunSetup.vue'
import DatabaseUnreachable from '@/components/setup/DatabaseUnreachable.vue'
import AppLogo from '@/components/ui/AppLogo.vue'
import { fetchDatabaseStatus } from '@/api/settings.api'
import { useConsoleStore } from '@/stores/console.store'
import type { DatabaseStatus } from '@/model/settings'

/**
 * Decides, once, which of three things the screen mounts as: the routed application (console
 * plus the catalog pages beside it), the first-run wizard, or the recovery screen for a
 * database that has stopped being reachable.
 *
 * This is also where the catalog is loaded, and the only place it is: the console used to load
 * it in its own `onMounted`, which was fine while it was the only screen, but a deep link or a
 * refresh landing on `/projects/:id` never mounts the console at all.
 */
const store = useConsoleStore()
const status = ref<DatabaseStatus | null>(null)
const failed = ref(false)

onMounted(async () => {
  try {
    status.value = await fetchDatabaseStatus()
    if (status.value.status === 'READY') await store.load()
  } catch {
    failed.value = true
  }
})
</script>

<template>
  <template v-if="status">
    <router-view v-if="status.status === 'READY'" />
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
