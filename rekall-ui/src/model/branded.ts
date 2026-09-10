type Brand<T, B extends string> = T & { readonly __brand: B }

export type CompanyId = Brand<string, 'CompanyId'>
export type ProjectId = Brand<string, 'ProjectId'>
export type TaskId = Brand<string, 'TaskId'>
export type TaskStepId = Brand<string, 'TaskStepId'>
export type DocumentId = Brand<string, 'DocumentId'>
export type WrapupId = Brand<string, 'WrapupId'>
export type TimeEntryId = Brand<string, 'TimeEntryId'>
export type TerminalId = Brand<string, 'TerminalId'>

export const asCompanyId = (value: string): CompanyId => value as CompanyId
export const asProjectId = (value: string): ProjectId => value as ProjectId
export const asTaskId = (value: string): TaskId => value as TaskId
export const asTaskStepId = (value: string): TaskStepId => value as TaskStepId
export const asDocumentId = (value: string): DocumentId => value as DocumentId
export const asWrapupId = (value: string): WrapupId => value as WrapupId
export const asTimeEntryId = (value: string): TimeEntryId => value as TimeEntryId
export const asTerminalId = (value: string): TerminalId => value as TerminalId
