import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import * as schemaApi from '@/api/schema.api'
import type { CreateEntityInput, CreateRelationInput, FieldInput, UpdateEntityInput } from '@/api/schema.api'
import type { Entity, Relation } from '@/model/schema'
import type { EntityId, EntityName, FieldId, RelationId } from '@/model/branded'

/**
 * The meta-model, held once for the whole application.
 *
 * Every screen needs it and it changes rarely, so refetching per route would put several
 * requests in front of each navigation for data that has not moved.
 */
export const useSchemaStore = defineStore('schema', () => {
  const entities = ref<readonly Entity[]>([])
  const relations = ref<readonly Relation[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const appliedEntities = computed(() => entities.value.filter((entity) => entity.status !== 'DRAFT'))

  const byName = computed(
    () => (name: EntityName) => entities.value.find((entity) => entity.physicalName === name) ?? null
  )

  const byId = computed(() => (id: EntityId) => entities.value.find((entity) => entity.id === id) ?? null)

  const outgoingRelations = computed(
    () => (id: EntityId) => relations.value.filter((relation) => relation.sourceTableId === id)
  )

  const incomingRelations = computed(
    () => (id: EntityId) =>
      relations.value.filter(
        (relation) => relation.targetTableId === id && relation.sourceTableId !== id
      )
  )

  async function load(): Promise<void> {
    isLoading.value = true
    error.value = null
    try {
      const [loadedEntities, loadedRelations] = await Promise.all([
        schemaApi.fetchEntities(),
        schemaApi.fetchRelations()
      ])
      entities.value = loadedEntities
      relations.value = loadedRelations
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Could not load the schema'
      throw e
    } finally {
      isLoading.value = false
    }
  }

  async function createEntity(input: CreateEntityInput): Promise<Entity> {
    const created = await schemaApi.createEntity(input)
    await load()
    return created
  }

  async function updateEntity(id: EntityId, input: UpdateEntityInput): Promise<void> {
    await schemaApi.updateEntity(id, input)
    await load()
  }

  async function removeEntity(id: EntityId): Promise<void> {
    await schemaApi.deleteEntity(id)
    await load()
  }

  async function addField(entityId: EntityId, input: FieldInput): Promise<void> {
    await schemaApi.addField(entityId, input)
    await load()
  }

  async function removeField(fieldId: FieldId): Promise<void> {
    await schemaApi.deleteField(fieldId)
    await load()
  }

  async function createRelation(input: CreateRelationInput): Promise<void> {
    await schemaApi.createRelation(input)
    await load()
  }

  async function removeRelation(id: RelationId): Promise<void> {
    await schemaApi.deleteRelation(id)
    await load()
  }

  return {
    entities,
    relations,
    isLoading,
    error,
    appliedEntities,
    byName,
    byId,
    outgoingRelations,
    incomingRelations,
    load,
    createEntity,
    updateEntity,
    removeEntity,
    addField,
    removeField,
    createRelation,
    removeRelation
  }
})
