import { describe, expect, it } from 'vitest'
import { locationFromQuery, locationToQuery, type ConsoleLocation } from '@/model/console-location'
import type { DocumentId, TaskId } from '@/model/branded'

const onSteps: ConsoleLocation = {
  mode: 'tasks',
  taskId: 't1' as TaskId,
  documentId: 'd1' as DocumentId,
  pane: 'steps'
}

describe('the console location in the URL', () => {
  it('reads back what it wrote', () => {
    const query = locationToQuery(onSteps)

    expect(query).toEqual({ mode: 'tasks', task: 't1', note: 'd1', pane: 'steps' })
    expect(locationFromQuery({ mode: 'tasks', task: 't1', note: 'd1', pane: 'steps' })).toEqual(onSteps)
  })

  it('leaves out the note pane and empty selections, but always writes the mode', () => {
    const query = locationToQuery({ mode: 'notes', taskId: null, documentId: null, pane: 'note' })

    expect(query).toEqual({ mode: 'notes', task: undefined, note: undefined, pane: undefined })
  })

  it('reads a bare link as no location at all', () => {
    expect(locationFromQuery({})).toBeNull()
    expect(locationFromQuery({ mode: 'elsewhere', task: 't1' })).toBeNull()
  })

  it('takes an unknown pane as the note and the first of a repeated value', () => {
    expect(locationFromQuery({ mode: 'tasks', task: ['t1', 't2'], pane: 'bogus' })).toEqual({
      mode: 'tasks',
      taskId: 't1',
      documentId: null,
      pane: 'note'
    })
  })
})
