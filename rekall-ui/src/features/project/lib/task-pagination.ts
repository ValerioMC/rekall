export interface PageOfItems<Item> {
  readonly items: readonly Item[]
  readonly page: number
  readonly pageCount: number
  readonly total: number
  readonly from: number
  readonly to: number
}

type Weight<Item> = (item: Item) => number

function totalWeight<Item>(items: readonly Item[], weightOf: Weight<Item>): number {
  return items.reduce((sum, item) => sum + weightOf(item), 0)
}

/** Items in pages of at most `size` rows. An item is never split, so one heavier than a page has a page to itself. */
function pagesOf<Item>(items: readonly Item[], size: number, weightOf: Weight<Item>): Item[][] {
  const pages: Item[][] = [[]]
  let used = 0
  for (const item of items) {
    const weight = weightOf(item)
    if (used > 0 && used + weight > size) {
      pages.push([])
      used = 0
    }
    pages[pages.length - 1]!.push(item)
    used += weight
  }
  return pages
}

/** The page asked for, held between the first and the last one that has rows. */
export function pageOf<Item>(
  items: readonly Item[],
  requested: number,
  size: number,
  weightOf: Weight<Item> = () => 1
): PageOfItems<Item> {
  const pages = pagesOf(items, size, weightOf)
  const page = Math.min(Math.max(1, requested), pages.length)
  const shown = pages[page - 1]!
  const before = totalWeight(pages.slice(0, page - 1).flat(), weightOf)
  const onPage = totalWeight(shown, weightOf)
  return {
    items: shown,
    page,
    pageCount: pages.length,
    total: totalWeight(items, weightOf),
    from: onPage === 0 ? 0 : before + 1,
    to: before + onPage
  }
}

/** The numbers a pager offers: the first, the last and the current with its neighbours; `null` stands for a gap. */
export function pageNumbers(current: number, count: number): (number | null)[] {
  const offered = [...new Set([1, current - 1, current, current + 1, count])]
    .filter((page) => page >= 1 && page <= count)
    .sort((a, b) => a - b)
  return offered.flatMap((page, index) => (index > 0 && page - offered[index - 1]! > 1 ? [null, page] : [page]))
}
