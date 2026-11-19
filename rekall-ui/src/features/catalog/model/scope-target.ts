import type { CompanyId, ProjectId } from '@/model/branded'

/** What the project switcher can hand back: everything, one company's work, or one project. */
export type ScopeTarget =
  | Readonly<{ kind: 'all' }>
  | Readonly<{ kind: 'company'; companyId: CompanyId }>
  | Readonly<{ kind: 'project'; companyId: CompanyId; projectId: ProjectId }>
