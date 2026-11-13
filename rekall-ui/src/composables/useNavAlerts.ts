import { useConsoleStore } from '@/stores/console.store'
import { useVulnerabilityStore } from '@/features/vulnerability/store/vulnerability.store'
import type { NavAlert } from '@/model/nav-tab'

/** The count each menu alert stands for, read from the store that owns it. */
export function useNavAlerts(): (alert: NavAlert | undefined) => number {
  const vulnerabilities = useVulnerabilityStore()
  const consoleStore = useConsoleStore()
  return (alert) => {
    if (alert === 'open-vulnerabilities') return vulnerabilities.openWarnings
    if (alert === 'review-queue') return consoleStore.reviewQueue.length
    return 0
  }
}
