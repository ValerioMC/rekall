import { mermaidSourcesOf } from '@/features/export/lib/diagram-images'
import type { ExportSelection } from '@/features/export/model/project-export'
import type { ProjectComponent } from '@/features/project/model/project-component'
import type { ProjectSection } from '@/features/project/model/project-section'
import type { SectionContext } from '@/features/project/model/section-context'

export interface PageGroup {
  readonly context: SectionContext
  readonly pages: readonly ProjectSection[]
}

export function pageGroupsOf(contexts: readonly SectionContext[], sections: readonly ProjectSection[]): PageGroup[] {
  return contexts
    .map((context) => ({ context, pages: sections.filter((section) => section.contextId === context.id) }))
    .filter((group) => group.pages.length > 0)
}

export function selectionSize(selection: ExportSelection): number {
  return (selection.blueprint ? 1 : 0) + selection.sectionIds.length + selection.componentIds.length
}

/** The markdown of everything ticked: the only text whose diagrams have to be drawn. */
export function tickedMarkdown(
  blueprint: string | null,
  sections: readonly ProjectSection[],
  components: readonly ProjectComponent[],
  selection: ExportSelection
): string[] {
  const pages = sections.filter((section) => selection.sectionIds.includes(section.id)).map((section) => section.bodyMarkdown)
  const parts = components.filter((component) => selection.componentIds.includes(component.id)).map((component) => component.description)
  return [...(selection.blueprint && blueprint !== null ? [blueprint] : []), ...pages, ...parts]
}

export function diagramSourcesOf(markdowns: readonly string[]): string[] {
  return [...new Set(markdowns.flatMap(mermaidSourcesOf))]
}
