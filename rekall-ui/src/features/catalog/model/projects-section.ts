export type ProjectsSection = 'projects' | 'companies' | 'connections'

export const PROJECTS_SECTIONS: readonly Readonly<{ id: ProjectsSection; label: string; to: string }>[] = [
  { id: 'projects', label: 'Projects', to: '/projects' },
  { id: 'companies', label: 'Companies', to: '/companies' },
  { id: 'connections', label: 'Connections', to: '/connections' }
]
