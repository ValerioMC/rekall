import type { CompanyId, ProjectId } from '@/model/branded'

/** What the project switcher hands back: the project chosen and the company it belongs to. */
export type ScopeTarget = Readonly<{ companyId: CompanyId; projectId: ProjectId }>
