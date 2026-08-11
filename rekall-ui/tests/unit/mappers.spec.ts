import { describe, expect, it } from 'vitest'
import { previewFields, splitList, toCellText, toFormModel, toPhysicalName } from '@/model/mappers'
import { asEntityName, asFieldId, asRecordId } from '@/model/branded'
import type { EntityField } from '@/model/schema'
import type { EntityRecord } from '@/model/records'

function field(overrides: Partial<EntityField> & Pick<EntityField, 'columnName' | 'type'>): EntityField {
  return {
    id: asFieldId('00000000-0000-0000-0000-000000000001'),
    label: overrides.columnName,
    description: '',
    nullable: true,
    defaultValue: null,
    length: null,
    precision: null,
    scale: null,
    enumValues: [],
    position: 0,
    ...overrides
  }
}

describe('toPhysicalName', () => {
  it('turns a label into a valid identifier', () => {
    expect(toPhysicalName('Project')).toBe('project')
    expect(toPhysicalName('Code Validator')).toBe('code_validator')
    expect(toPhysicalName('  Ambienti / Cluster ')).toBe('ambienti_cluster')
  })

  it('strips accents rather than emitting characters Postgres would need quoted', () => {
    expect(toPhysicalName('Attività')).toBe('attivita')
  })

  it('never exceeds the 63 byte identifier limit', () => {
    expect(toPhysicalName('a'.repeat(120))).toHaveLength(63)
  })
})

describe('splitList', () => {
  it('trims and drops empty entries', () => {
    expect(splitList(' progetti , commesse , ')).toEqual(['progetti', 'commesse'])
    expect(splitList('')).toEqual([])
  })
})

describe('toCellText', () => {
  const target: EntityRecord = {
    id: asRecordId('00000000-0000-0000-0000-0000000000aa'),
    entityName: asEntityName('environment'),
    label: 'kmaster14 / stvv-dev',
    values: {},
    createdAt: new Date(),
    updatedAt: new Date()
  }

  it('shows a resolved reference by its label, never as a uuid', () => {
    expect(toCellText(target)).toBe('kmaster14 / stvv-dev')
  })

  it('joins tags and renders empty values as nothing', () => {
    expect(toCellText(['esa', 'backend'])).toBe('esa, backend')
    expect(toCellText(null)).toBe('')
  })
})

describe('toFormModel', () => {
  it('flattens a resolved reference back to the id the form edits', () => {
    const environment: EntityRecord = {
      id: asRecordId('00000000-0000-0000-0000-0000000000aa'),
      entityName: asEntityName('environment'),
      label: 'kmaster14',
      values: {},
      createdAt: new Date(),
      updatedAt: new Date()
    }
    const task: EntityRecord = {
      id: asRecordId('00000000-0000-0000-0000-0000000000bb'),
      entityName: asEntityName('task'),
      label: 'code-validator',
      values: { name: 'code-validator', environment_id: environment },
      createdAt: new Date(),
      updatedAt: new Date()
    }

    expect(toFormModel(task)).toEqual({
      name: 'code-validator',
      environment_id: environment.id
    })
  })
})

describe('previewFields', () => {
  it('keeps long text out of a table and caps the column count', () => {
    const fields = [
      field({ columnName: 'name', type: 'TEXT' }),
      field({ columnName: 'notes', type: 'LONG_TEXT' }),
      field({ columnName: 'body', type: 'MARKDOWN' }),
      field({ columnName: 'status', type: 'ENUM' }),
      field({ columnName: 'a', type: 'TEXT' }),
      field({ columnName: 'b', type: 'TEXT' }),
      field({ columnName: 'c', type: 'TEXT' })
    ]

    const preview = previewFields(fields)

    expect(preview).toHaveLength(4)
    expect(preview.map((f) => f.columnName)).not.toContain('notes')
    expect(preview.map((f) => f.columnName)).not.toContain('body')
  })
})
