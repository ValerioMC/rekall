import type { CompanyId, ProjectId } from '../../../model/branded'

/**
 * Where a note lives. The scope owns it: the note is listed there and stays there when it is
 * taken off its last task. Sessions still read a note only through the tasks it is on.
 */
export type NoteScope =
  | Readonly<{ kind: 'GLOBAL' }>
  | Readonly<{ kind: 'COMPANY'; id: CompanyId }>
  | Readonly<{ kind: 'PROJECT'; id: ProjectId }>

export const GLOBAL_SCOPE: NoteScope = { kind: 'GLOBAL' }
