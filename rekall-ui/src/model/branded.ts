type Brand<T, B extends string> = T & { readonly __brand: B }

export type CompanyId = Brand<string, 'CompanyId'>
export type ProjectId = Brand<string, 'ProjectId'>
export type TaskId = Brand<string, 'TaskId'>
export type TaskStepId = Brand<string, 'TaskStepId'>
export type StepLinkId = Brand<string, 'StepLinkId'>
export type StepVisitId = Brand<string, 'StepVisitId'>
export type DocumentId = Brand<string, 'DocumentId'>
export type WrapupId = Brand<string, 'WrapupId'>
export type TimeEntryId = Brand<string, 'TimeEntryId'>
export type TerminalId = Brand<string, 'TerminalId'>
export type TagId = Brand<string, 'TagId'>
export type DiagramId = Brand<string, 'DiagramId'>
export type ProjectSectionId = Brand<string, 'ProjectSectionId'>
export type ProjectComponentId = Brand<string, 'ProjectComponentId'>
export type ConnectionId = Brand<string, 'ConnectionId'>

export const asCompanyId = (value: string): CompanyId => value as CompanyId
export const asProjectId = (value: string): ProjectId => value as ProjectId
export const asTaskId = (value: string): TaskId => value as TaskId
export const asTaskStepId = (value: string): TaskStepId => value as TaskStepId
export const asStepLinkId = (value: string): StepLinkId => value as StepLinkId
export const asStepVisitId = (value: string): StepVisitId => value as StepVisitId
export const asDocumentId = (value: string): DocumentId => value as DocumentId
export const asWrapupId = (value: string): WrapupId => value as WrapupId
export const asTimeEntryId = (value: string): TimeEntryId => value as TimeEntryId
export const asTerminalId = (value: string): TerminalId => value as TerminalId
export const asTagId = (value: string): TagId => value as TagId
export const asDiagramId = (value: string): DiagramId => value as DiagramId
export const asProjectSectionId = (value: string): ProjectSectionId => value as ProjectSectionId
export const asProjectComponentId = (value: string): ProjectComponentId => value as ProjectComponentId
export const asConnectionId = (value: string): ConnectionId => value as ConnectionId
