/**
 * Branded identifiers.
 *
 * Every id in Rekall is a uuid string, so without brands a project id, a task id and a
 * document id are the same type and the compiler cannot tell you when they are swapped. That
 * mistake is silent at runtime too: the request simply returns nothing.
 */
type Brand<T, B extends string> = T & { readonly __brand: B }

export type ProjectId = Brand<string, 'ProjectId'>
export type TaskId = Brand<string, 'TaskId'>
export type EnvironmentId = Brand<string, 'EnvironmentId'>
export type DocumentId = Brand<string, 'DocumentId'>

export const asProjectId = (value: string): ProjectId => value as ProjectId
export const asTaskId = (value: string): TaskId => value as TaskId
export const asEnvironmentId = (value: string): EnvironmentId => value as EnvironmentId
export const asDocumentId = (value: string): DocumentId => value as DocumentId
