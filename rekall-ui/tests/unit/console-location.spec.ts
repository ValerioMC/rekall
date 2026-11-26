import { describe, expect, it } from 'vitest'
import { locationFromQuery, locationToQuery, type ConsoleLocation } from '@/model/console-location'
import type { DocumentId, TaskId, TerminalId } from '@/model/branded'

const onSteps: ConsoleLocation = {
  taskId: 't1' as TaskId,
  documentId: 'd1' as DocumentId,
  pane: 'steps',
  deviationStep: null,
  chatId: null
}

describe('the console location in the URL', () => {
  it('reads back what it wrote', () => {
    const query = locationToQuery(onSteps)

    expect(query).toEqual({ mode: 'tasks', task: 't1', note: 'd1', pane: 'steps', deviation: undefined })
    expect(locationFromQuery({ mode: 'tasks', task: 't1', note: 'd1', pane: 'steps' })).toEqual(onSteps)
  })

  it('leaves out the note pane and empty selections, but always writes the mark', () => {
    const query = locationToQuery({ taskId: null, documentId: null, pane: 'note', deviationStep: null, chatId: null })

    expect(query).toEqual({ mode: 'tasks', task: undefined, note: undefined, pane: undefined, deviation: undefined })
  })

  it('reads a bare link as no location at all', () => {
    expect(locationFromQuery({})).toBeNull()
    expect(locationFromQuery({ mode: 'elsewhere', task: 't1' })).toBeNull()
    expect(locationFromQuery({ mode: 'notes', task: 't1' })).toBeNull()
  })

  it('takes an unknown pane as the note and the first of a repeated value', () => {
    expect(locationFromQuery({ mode: 'tasks', task: ['t1', 't2'], pane: 'bogus' })).toEqual({
      taskId: 't1',
      documentId: null,
      pane: 'note',
      deviationStep: null,
      chatId: null
    })
  })

  it('names a task\'s chat by the task, and reads it back', () => {
    const onChat: ConsoleLocation = { ...onSteps, documentId: null, pane: 'chat' }
    const query = locationToQuery(onChat)

    expect(query).toEqual({ mode: 'tasks', task: 't1', note: undefined, pane: 'chat', deviation: undefined, chat: undefined })
    expect(locationFromQuery({ mode: 'tasks', task: 't1', pane: 'chat' })).toEqual(onChat)
  })

  it('names a chat on no task by its session, and reads it back', () => {
    const onChat: ConsoleLocation = { taskId: null, documentId: null, pane: 'chat', deviationStep: null, chatId: 'chat-9' as TerminalId }
    const query = locationToQuery(onChat)

    expect(query).toMatchObject({ pane: 'chat', task: undefined, chat: 'chat-9' })
    expect(locationFromQuery({ mode: 'tasks', pane: 'chat', chat: 'chat-9' })).toEqual(onChat)
  })

  it('writes the step of the deviation in view and reads it back, refusing a step that is not a positive number', () => {
    const reading: ConsoleLocation = { ...onSteps, pane: 'deviation', deviationStep: 3 }

    expect(locationToQuery(reading)).toMatchObject({ pane: 'deviation', deviation: '3' })
    expect(locationFromQuery({ mode: 'tasks', task: 't1', note: 'd1', pane: 'deviation', deviation: '3' })).toEqual(reading)
    expect(locationFromQuery({ mode: 'tasks', task: 't1', pane: 'deviation', deviation: 'x' })?.deviationStep).toBeNull()
    expect(locationFromQuery({ mode: 'tasks', task: 't1', pane: 'deviation', deviation: '0' })?.deviationStep).toBeNull()
  })
})
