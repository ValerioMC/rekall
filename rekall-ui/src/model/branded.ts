/**
 * Branded identifiers.
 *
 * Every id in Rekall is a uuid string, so without brands a project id, a task id and a
 * document id are the same type and the compiler cannot tell you when they are swapped. That
 * mistake is silent at runtime too: the request simply returns nothing.
 */
type Brand<T, B extends string> = T & { readonly __brand: B }

export type CompanyId = Brand<string, 'CompanyId'>
export type ProjectId = Brand<string, 'ProjectId'>
export type TaskId = Brand<string, 'TaskId'>
export type DocumentId = Brand<string, 'DocumentId'>
export type WrapupId = Brand<string, 'WrapupId'>
export type TimeEntryId = Brand<string, 'TimeEntryId'>

export const asCompanyId = (value: string): CompanyId => value as CompanyId
export const asProjectId = (value: string): ProjectId => value as ProjectId
export const asTaskId = (value: string): TaskId => value as TaskId
export const asDocumentId = (value: string): DocumentId => value as DocumentId
export const asWrapupId = (value: string): WrapupId => value as WrapupId
export const asTimeEntryId = (value: string): TimeEntryId => value as TimeEntryId
