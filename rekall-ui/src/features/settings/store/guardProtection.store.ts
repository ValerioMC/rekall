import { defineStore } from 'pinia'
import { ref } from 'vue'
import {
  fetchGuardProtection,
  installGuardProtection,
  removeGuardProtection
} from '@/features/settings/api/guardProtection.api'
import type { GuardProtection } from '@/features/settings/model/guardProtection'

/** Whether Claude Code runs Rekall's guard hooks, as the server reads it from `settings.json`. */
export const useGuardProtectionStore = defineStore('guardProtection', () => {
  const protection = ref<GuardProtection | null>(null)

  async function load(): Promise<void> {
    protection.value = await fetchGuardProtection()
  }

  async function install(): Promise<void> {
    protection.value = await installGuardProtection()
  }

  async function remove(): Promise<void> {
    protection.value = await removeGuardProtection()
  }

  return { protection, load, install, remove }
})
