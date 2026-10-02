<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { fetchVersionStatus } from '@/api/version.api'
import { isDesktopApp } from '@/common/native/desktop'
import type { VersionStatus } from '@/model/version'

const status = ref<VersionStatus | null>(null)

const update = computed(() => (status.value?.check === 'UPDATE_AVAILABLE' ? status.value.latest : null))
const updateHref = computed(() => update.value?.downloadUrl ?? update.value?.releaseUrl ?? null)
// The desktop window hands a link off the local server to the browser on its own; a browser tab
// would leave the console, so it opens a new one.
const opensNewTab = !isDesktopApp()

// A failed check leaves the label out: the version is a convenience, never a reason to nag.
onMounted(async () => {
  try {
    status.value = await fetchVersionStatus()
  } catch {
    status.value = null
  }
})
</script>

<template>
  <span v-if="status" class="flex items-baseline gap-1.5" data-testid="app-version">
    <span class="font-mono text-[10px] leading-tight text-text-subtle">v{{ status.current }}</span>
    <a
      v-if="update && updateHref"
      :href="updateHref"
      :target="opensNewTab ? '_blank' : undefined"
      :rel="opensNewTab ? 'noopener noreferrer' : undefined"
      :title="`Download Rekall ${update.version}`"
      class="focus-ring rounded-[3px] bg-accent-soft px-1 font-mono text-[10px] font-semibold leading-tight text-accent hover:underline"
      data-testid="app-update"
    >
      v{{ update.version }} available
    </a>
  </span>
</template>
