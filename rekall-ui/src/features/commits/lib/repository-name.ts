export const PROJECT_FOLDER_NAME = 'Project folder'

/** What a repository is called when only its folder is known: the folder's own name. */
export function repositoryNameOf(folder: string | null): string {
  return (folder ?? '').split('/').filter(Boolean).pop() ?? PROJECT_FOLDER_NAME
}
