import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { fetchVersionStatus } from '@/api/version.api'
import { canInstallUpdate, installUpdate, isDesktopApp } from '@/common/native/desktop'
import type { LatestRelease, VersionStatus } from '@/model/version'

/**
 * This build's version and what GitHub says about a newer one. The desktop window checks once at
 * startup and opens `UpdateDialog` only when there is something to install; Settings checks by hand.
 */
export const useUpdateStore = defineStore('update', () => {
  const status = ref<VersionStatus | null>(null)
  const checking = ref(false)
  const installing = ref(false)
  const installFailure = ref<string | null>(null)
  const promptOpen = ref(false)
  let startupChecked = false

  const latest = computed<LatestRelease | null>(() =>
    status.value?.check === 'UPDATE_AVAILABLE' ? status.value.latest : null
  )
  const downloadHref = computed(() => latest.value?.downloadUrl ?? latest.value?.releaseUrl ?? null)
  /** False after a failed install too: the browser download is the way out. */
  const installsItself = computed(() => canInstallUpdate() && installFailure.value === null)

  async function check(refresh = false): Promise<VersionStatus> {
    checking.value = true
    try {
      status.value = await fetchVersionStatus(refresh)
      return status.value
    } finally {
      checking.value = false
    }
  }

  /** Silent when offline or up to date: Settings can ask again by hand. */
  async function checkAtStartup(): Promise<void> {
    if (startupChecked || !isDesktopApp()) return
    startupChecked = true
    try {
      await check()
      promptOpen.value = latest.value !== null
    } catch {
      promptOpen.value = false
    }
  }

  /** Installs and restarts; resolves without restarting when the user kept a live session. */
  async function install(): Promise<void> {
    installing.value = true
    installFailure.value = null
    try {
      if (await installUpdate()) return
      promptOpen.value = false
    } catch (failure) {
      installFailure.value = failure instanceof Error ? failure.message : String(failure)
    }
    installing.value = false
  }

  function dismiss(): void {
    if (!installing.value) promptOpen.value = false
  }

  return {
    status,
    checking,
    installing,
    installFailure,
    promptOpen,
    latest,
    downloadHref,
    installsItself,
    check,
    checkAtStartup,
    install,
    dismiss
  }
})
