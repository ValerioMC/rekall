/** Scrolls to the `h2` or `h3` of `article` that reads `text`, without animation when the person asks for less motion. */
export function jumpToHeading(article: HTMLElement | null, text: string): void {
  const target = [...(article?.querySelectorAll('h2, h3') ?? [])].find((heading) => heading.textContent?.trim() === text)
  const smooth = !window.matchMedia('(prefers-reduced-motion: reduce)').matches
  target?.scrollIntoView({ behavior: smooth ? 'smooth' : 'auto', block: 'start' })
}
