<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { fetchVersionStatus } from '@/api/version.api'
import { canInstallUpdate, isDesktopApp } from '@/common/native/desktop'
import UpdateDialog from '@/components/shell/UpdateDialog.vue'
import type { VersionStatus } from '@/model/version'

const status = ref<VersionStatus | null>(null)
const dialogOpen = ref(false)

const update = computed(() => (status.value?.check === 'UPDATE_AVAILABLE' ? status.value.latest : null))
const updateHref = computed(() => update.value?.downloadUrl ?? update.value?.releaseUrl ?? null)
// The desktop window hands a link off the local server to the browser on its own; a browser tab
// would leave the console, so it opens a new one.
const opensNewTab = !isDesktopApp()
// The macOS window installs the release itself, so the file never gets the quarantine flag.
const installsItself = canInstallUpdate()

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
    <button
      v-if="update && updateHref && installsItself"
      type="button"
      :title="`Install Rekall ${update.version}`"
      class="focus-ring rounded-[3px] bg-accent-soft px-1 font-mono text-[10px] font-semibold leading-tight text-accent hover:underline"
      data-testid="app-update"
      @click="dialogOpen = true"
    >
      v{{ update.version }} available
    </button>
    <a
      v-else-if="update && updateHref"
      :href="updateHref"
      :target="opensNewTab ? '_blank' : undefined"
      :rel="opensNewTab ? 'noopener noreferrer' : undefined"
      :title="`Download Rekall ${update.version}`"
      class="focus-ring rounded-[3px] bg-accent-soft px-1 font-mono text-[10px] font-semibold leading-tight text-accent hover:underline"
      data-testid="app-update"
    >
      v{{ update.version }} available
    </a>
    <!-- Out of the header: its backdrop filter would make the dialog's fixed overlay its own. -->
    <Teleport to="body">
      <UpdateDialog
        v-if="dialogOpen && update && updateHref"
        :version="update.version"
        :download-href="updateHref"
        @close="dialogOpen = false"
      />
    </Teleport>
  </span>
</template>
