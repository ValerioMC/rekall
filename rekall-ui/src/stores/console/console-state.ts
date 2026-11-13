import { ref } from 'vue'
import type { CommitReference } from '@/features/commits/model/commitReference'
import type { Company, Project, Tag, Task } from '@/features/catalog/model/catalog'
import type { RekallDocument } from '@/features/notes/model/document'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { TimeEntry } from '@/features/time/model/time-entry'
import type { Wrapup } from '@/features/wrapup/model/wrapup'
import type { TagSkill, TaskSkill } from '@/features/skills/model/skill-assignment'
import type { CompanyId, DocumentId, ProjectId, TaskId, TaskStepId } from '@/model/branded'
import type { PaneFocus } from '@/model/console-location'

export type SaveState = 'saved' | 'unsaved' | 'saving'

/** The rows the console holds and where the reader is in them. Every slice of the store reads and writes these. */
export function createConsoleState() {
  return {
    companies: ref<Company[]>([]),
    projects: ref<Project[]>([]),
    tasks: ref<Task[]>([]),
    tags: ref<Tag[]>([]),
    documents: ref<RekallDocument[]>([]),
    wrapups: ref<Wrapup[]>([]),
    steps: ref<TaskStep[]>([]),
    timeEntries: ref<TimeEntry[]>([]),
    commitReferences: ref<CommitReference[]>([]),
    taskSkills: ref<TaskSkill[]>([]),
    tagSkills: ref<TagSkill[]>([]),

    isLoading: ref(true),
    saveState: ref<SaveState>('saved'),

    scopeCompany: ref<CompanyId | null>(null),
    scopeProject: ref<ProjectId | null>(null),
    /** Quick tasks stay out of the lists while this is on; they are shown by default. */
    hideQuickTasks: ref(false),
    selectedTaskId: ref<TaskId | null>(null),
    selectedDocId: ref<DocumentId | null>(null),
    paneFocus: ref<PaneFocus>('note'),
    /** The step whose deviation the deviation pane reads. */
    selectedDeviationStep: ref<number | null>(null),
    /** A step something outside the steps pane asked to see; the pane opens it and clears this. */
    stepToOpen: ref<TaskStepId | null>(null)
  }
}

export type ConsoleState = ReturnType<typeof createConsoleState>
