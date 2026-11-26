import { defineStore } from 'pinia'
import { ref } from 'vue'
import {
  fetchGuardProtection,
  installGuardProtection,
  removeGuardProtection
} from '@/features/settings/api/guardProtection.api'
import type { GuardProtection } from '@/features/settings/model/guardProtection'
import type { SessionProvider } from '@/features/claude/model/session-provider'

export const useGuardProtectionStore = defineStore('guardProtection', () => {
  const byProvider = ref<Partial<Record<SessionProvider, GuardProtection>>>({})

  async function load(provider: SessionProvider): Promise<void> {
    byProvider.value[provider] = await fetchGuardProtection(provider)
  }

  async function install(provider: SessionProvider): Promise<void> {
    byProvider.value[provider] = await installGuardProtection(provider)
  }

  async function remove(provider: SessionProvider): Promise<void> {
    byProvider.value[provider] = await removeGuardProtection(provider)
  }

  return { byProvider, load, install, remove }
})
