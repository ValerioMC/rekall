import { useConsoleStore } from '@/stores/console.store'
import type { RecordDraft } from '@/model/record-draft'
import type { ProjectId } from '@/model/branded'

type Draft<Kind extends RecordDraft['kind']> = Extract<RecordDraft, { kind: Kind }>

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

  async function writeCompany(draft: Draft<'company'>): Promise<void> {
    const input = { name: draft.name.trim(), description: descriptionOf(draft) }
    if (draft.id === null) await store.createCompany(input)
    else await store.updateCompany(draft.id, input)
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
    if (draft.id === null) await store.createTask(input)
    else await store.updateTask(draft.id, input)
  }

  /** Creates or updates the record; `afterProject` runs once a project has its id. */
  async function write(draft: RecordDraft, label: string, afterProject: (id: ProjectId) => Promise<void>): Promise<void> {
    if (draft.kind === 'company') await writeCompany(draft)
    else if (draft.kind === 'project') await afterProject(await writeProject(draft, label))
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
