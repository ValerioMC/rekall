import {
  createCompany as apiCreateCompany,
  deleteCompany as apiDeleteCompany,
  removeCompanyLogoImage as apiRemoveCompanyLogoImage,
  updateCompany as apiUpdateCompany,
  uploadCompanyLogoImage as apiUploadCompanyLogoImage
} from '@/api/catalog.api'
import type { CompanyInput } from '@/api/catalog.api'
import type { Company } from '@/model/catalog'
import type { CompanyId } from '@/model/branded'
import type { ConsoleState } from './console-state'
import type { useConsoleNavigation } from './useConsoleNavigation'
import type { useConsoleRefreshes } from './useConsoleRefreshes'
import { replaceById } from './rows'

/** A company created comes into scope; one renamed is re-read under its projects and tasks, which show its name. */
export function useCompanyWrites(
  state: ConsoleState,
  navigation: ReturnType<typeof useConsoleNavigation>,
  refreshes: ReturnType<typeof useConsoleRefreshes>
) {
  const { companies, scopeCompany } = state

  async function createCompany(input: CompanyInput): Promise<Company> {
    const created = await apiCreateCompany(input)
    companies.value = [...companies.value, created].sort((a, b) => a.name.localeCompare(b.name))
    navigation.setScope(created.id)
    return created
  }

  async function updateCompany(id: CompanyId, input: CompanyInput): Promise<void> {
    const saved = await apiUpdateCompany(id, input)
    companies.value = replaceById(companies.value, saved).sort((a, b) => a.name.localeCompare(b.name))
    await Promise.all([refreshes.refreshProjects(), refreshes.refreshTasks()])
  }

  async function deleteCompany(id: CompanyId): Promise<void> {
    await apiDeleteCompany(id)
    if (scopeCompany.value === id) navigation.setScope(null)
    await refreshes.load()
  }

  /** `null` takes the logo off, and the company goes back to its initial. */
  async function setCompanyLogoImage(id: CompanyId, file: File | null): Promise<void> {
    if (file) {
      companies.value = replaceById(companies.value, await apiUploadCompanyLogoImage(id, file))
      return
    }
    await apiRemoveCompanyLogoImage(id)
    companies.value = companies.value.map((company) =>
      company.id === id ? { ...company, logoImageVersion: null } : company
    )
  }

  return { createCompany, updateCompany, deleteCompany, setCompanyLogoImage }
}
