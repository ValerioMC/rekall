import { useConsoleStore } from '@/stores/console.store'
import type { RecordDraft } from '@/model/record-draft'
import type { CompanyId, ProjectId } from '@/model/branded'

type Draft<Kind extends RecordDraft['kind']> = Extract<RecordDraft, { kind: Kind }>

/** What runs once a saved company or project has its id: its picture's upload or removal. */
export interface AfterSave {
  company: (id: CompanyId) => Promise<void>
  project: (id: ProjectId) => Promise<void>
}

/**
 * What saving or deleting a company, project or task from the record dialog sends to the store.
 * A project keeps the fields its own page edits (blueprint, folder, auto-commit) as they are
 * stored, since the dialog does not show them. A blank description is saved as none.
 */
export function useRecordWrites() {
  const store = useConsoleStore()

  function descriptionOf(draft: RecordDraft): string | null {
    const trimmed = draft.description.trim()
    return trimmed === '' ? null : trimmed
  }

  async function writeCompany(draft: Draft<'company'>): Promise<CompanyId> {
    const input = { name: draft.name.trim(), description: descriptionOf(draft) }
    if (draft.id === null) return (await store.createCompany(input)).id
    await store.updateCompany(draft.id, input)
    return draft.id
  }

  async function writeProject(draft: Draft<'project'>, label: string): Promise<ProjectId> {
    const stored = store.projects.find((candidate) => candidate.id === draft.id)
    const input = {
      label,
      title: draft.title.trim(),
      status: draft.status,
      icon: draft.icon,
      description: descriptionOf(draft),
      blueprintMarkdown: stored?.blueprintMarkdown ?? null,
      repoFolder: stored?.repoFolder ?? null,
      autoCommit: stored?.autoCommit ?? false,
      companyId: draft.companyId
    }
    if (draft.id === null) return (await store.createProject(input)).id
    await store.updateProject(draft.id, input)
    return draft.id
  }

  async function writeTask(draft: Draft<'task'>, label: string): Promise<void> {
    const input = {
      label,
      title: draft.title.trim(),
      status: draft.status,
      description: descriptionOf(draft),
      projectId: draft.projectId,
      tagId: draft.tagId
    }
    if (draft.id !== null) return store.updateTask(draft.id, input)
    const created = await store.createTask(input)
    if (draft.skills.length) await store.assignTaskSkills(created.id, draft.skills)
  }

  /** Creates or updates the record, then runs `after` for a company or a project. */
  async function write(draft: RecordDraft, label: string, after: AfterSave): Promise<void> {
    if (draft.kind === 'company') await after.company(await writeCompany(draft))
    else if (draft.kind === 'project') await after.project(await writeProject(draft, label))
    else await writeTask(draft, label)
  }

  async function erase(draft: RecordDraft): Promise<void> {
    if (draft.id === null) return
    if (draft.kind === 'company') await store.deleteCompany(draft.id)
    else if (draft.kind === 'project') await store.deleteProject(draft.id)
    else await store.deleteTask(draft.id)
  }

  /** What deleting the record takes with it, said before anyone confirms. */
  function deletionBlast(draft: RecordDraft): string {
    if (draft.kind === 'company') {
      const company = store.companies.find((candidate) => candidate.id === draft.id)
      return `${plural(company?.projectCount ?? 0, 'project')} · ${plural(company?.taskCount ?? 0, 'task')} · the notes it and its projects own`
    }
    if (draft.kind === 'project') {
      const count = store.tasks.filter((task) => task.projectId === draft.id).length
      return `${plural(count, 'task')} · the notes it owns`
    }
    const task = store.tasks.find((candidate) => candidate.id === draft.id)
    return `${plural(task?.documentCount ?? 0, 'note')} unlinked · each stays where it lives`
  }

  return { write, erase, deletionBlast }
}

function plural(count: number, noun: string): string {
  return `${count} ${noun}${count === 1 ? '' : 's'}`
}
