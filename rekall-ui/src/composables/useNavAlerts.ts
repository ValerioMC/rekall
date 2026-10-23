import { useVulnerabilityStore } from '@/features/vulnerability/store/vulnerability.store'
import type { NavAlert } from '@/model/nav-tab'

/** The count each tab alert stands for, read from the store that owns it. */
export function useNavAlerts(): (alert: NavAlert | undefined) => number {
  const vulnerabilities = useVulnerabilityStore()
  return (alert) => {
    if (alert === 'open-vulnerabilities') return vulnerabilities.openWarnings
    return 0
  }
}
