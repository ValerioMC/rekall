import { ref } from 'vue'
import type { CommitReference } from '@/features/commits/model/commitReference'
import type {
  Company,
  Project,
  RekallDocument,
  Tag,
  Task,
  TaskStep,
  TimeEntry,
  Wrapup
} from '@/features/catalog/model/catalog'
import type { TagSkill, TaskSkill } from '@/features/skills/model/skill-assignment'
import type { CompanyId, DocumentId, ProjectId, TaskId, TaskStepId } from '@/model/branded'
import type { NavMode, PaneFocus } from '@/model/console-location'

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
    navMode: ref<NavMode>('tasks'),
    filter: ref(''),
    selectedTaskId: ref<TaskId | null>(null),
    selectedDocId: ref<DocumentId | null>(null),
    paneFocus: ref<PaneFocus>('note'),
    /** A step something outside the steps pane asked to see; the pane opens it and clears this. */
    stepToOpen: ref<TaskStepId | null>(null),
    /** Browsing notes, the placements column gives way to the new-note composer while this is on. */
    noteComposerOpen: ref(false)
  }
}

export type ConsoleState = ReturnType<typeof createConsoleState>
