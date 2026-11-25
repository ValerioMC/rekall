import type { Project } from '@/features/catalog/model/catalog'
import type { ProjectExportEntry } from '@/features/export/model/project-export'
import { asCompanyId, asProjectComponentId, asProjectExportId, asProjectId, asProjectSectionId, asSectionContextId } from '@/model/branded'
import type { ProjectComponent } from '@/features/project/model/project-component'
import type { ProjectSection } from '@/features/project/model/project-section'
import type { SectionContext } from '@/features/project/model/section-context'

export const FLOW = 'flowchart LR\n  A --> B'

export const PROJECT = {
  id: asProjectId('11111111-1111-4111-8111-111111111111'),
  title: 'Vega Platform',
  companyName: 'VFORGE',
  blueprintMarkdown: `The blueprint.\n\n\`\`\`mermaid\n${FLOW}\n\`\`\``
} as Project

export const CONTEXT = { id: asSectionContextId('c0000000-0000-4000-8000-000000000001'), projectId: PROJECT.id, title: 'Architecture', position: 0 } as SectionContext

export const PAGE = { id: asProjectSectionId('a0000000-0000-4000-8000-000000000001'), projectId: PROJECT.id, contextId: CONTEXT.id, title: 'Layers', bodyMarkdown: 'Layers text', position: 0 } as ProjectSection

export const COMPONENT = { id: asProjectComponentId('b0000000-0000-4000-8000-000000000001'), projectId: PROJECT.id, name: 'rekall-core', folder: 'crates/rekall-core', summary: 'Core', description: 'Core text', position: 0 } as ProjectComponent

export function entryOf(overrides: Partial<ProjectExportEntry> = {}): ProjectExportEntry {
  return {
    id: asProjectExportId('e0000000-0000-4000-8000-000000000001'),
    projectId: PROJECT.id,
    projectLabel: 'vega',
    projectTitle: 'Vega Platform',
    companyId: asCompanyId('d0000000-0000-4000-8000-000000000001'),
    companyName: 'VFORGE',
    format: 'PDF',
    status: 'DONE',
    summary: 'Blueprint',
    fileName: 'vega-blueprint.pdf',
    sizeBytes: 188_416,
    diagramsMissing: 0,
    failureKind: null,
    failureMessage: null,
    createdAt: '2026-11-25T12:00:00Z',
    ...overrides
  }
}
