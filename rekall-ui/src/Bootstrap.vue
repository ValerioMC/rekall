<script setup lang="ts">
import { onMounted, ref } from 'vue'
import FirstRunSetup from '@/components/setup/FirstRunSetup.vue'
import DatabaseUnreachable from '@/components/setup/DatabaseUnreachable.vue'
import RunningTasksDock from '@/components/shell/RunningTasksDock.vue'
import AppLogo from '@/components/ui/AppLogo.vue'
import { fetchDatabaseStatus } from '@/api/settings.api'
import { useConsoleStore } from '@/stores/console.store'
import type { DatabaseStatus } from '@/model/settings'

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
    <template v-if="status.status === 'READY'">
      <router-view />
      <RunningTasksDock />
    </template>
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
