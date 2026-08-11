/**
 * Branded identifiers.
 *
 * Every id in Rekall is a uuid string, so without brands a table id, a field id and a record
 * id are the same type and the compiler cannot tell you when they are swapped. That mistake is
 * silent at runtime too: the request simply returns nothing.
 */
type Brand<T, B extends string> = T & { readonly __brand: B }

export type EntityId = Brand<string, 'EntityId'>
export type FieldId = Brand<string, 'FieldId'>
export type RelationId = Brand<string, 'RelationId'>
export type RecordId = Brand<string, 'RecordId'>
export type DocumentId = Brand<string, 'DocumentId'>
export type PlanId = Brand<string, 'PlanId'>

/** Physical table name, e.g. `project`. Used in urls, so it is worth keeping distinct. */
export type EntityName = Brand<string, 'EntityName'>

export const asEntityId = (value: string): EntityId => value as EntityId
export const asFieldId = (value: string): FieldId => value as FieldId
export const asRelationId = (value: string): RelationId => value as RelationId
export const asRecordId = (value: string): RecordId => value as RecordId
export const asDocumentId = (value: string): DocumentId => value as DocumentId
export const asEntityName = (value: string): EntityName => value as EntityName
