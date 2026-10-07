import type { CellAddress, TableOperation } from '@/common/live-preview/markdown-table'

/** What a right-click on a drawn table offers, and the table edit each choice makes. */

export type TableMenuAction =
  | 'rowAbove'
  | 'rowBelow'
  | 'columnLeft'
  | 'columnRight'
  | 'deleteRow'
  | 'deleteColumn'
  | 'alignLeft'
  | 'alignCenter'
  | 'alignRight'
  | 'editSource'

export interface TableMenuItem {
  action: TableMenuAction
  label: string
}

/** Groups, in the order the menu shows them. */
export const TABLE_MENU_GROUPS: TableMenuItem[][] = [
  [
    { action: 'rowAbove', label: 'Insert row above' },
    { action: 'rowBelow', label: 'Insert row below' },
    { action: 'columnLeft', label: 'Insert column left' },
    { action: 'columnRight', label: 'Insert column right' }
  ],
  [
    { action: 'deleteRow', label: 'Delete row' },
    { action: 'deleteColumn', label: 'Delete column' }
  ],
  [
    { action: 'alignLeft', label: 'Align left' },
    { action: 'alignCenter', label: 'Align center' },
    { action: 'alignRight', label: 'Align right' }
  ],
  [{ action: 'editSource', label: 'Edit as markdown' }]
]

/** The header row is neither inserted above nor deleted, and a table keeps at least one column. */
export function isTableActionAvailable(
  action: TableMenuAction,
  cell: CellAddress,
  columnCount: number
): boolean {
  if (action === 'rowAbove' || action === 'deleteRow') return cell.row > 0
  if (action === 'deleteColumn') return columnCount > 1
  return true
}

/** The edit a choice makes on the right-clicked cell's table; `editSource` makes none. */
export function tableOperationFor(
  action: Exclude<TableMenuAction, 'editSource'>,
  cell: CellAddress
): TableOperation {
  switch (action) {
    case 'rowAbove':
      return { kind: 'insertRow', row: cell.row - 1 }
    case 'rowBelow':
      return { kind: 'insertRow', row: cell.row }
    case 'columnLeft':
      return { kind: 'insertColumn', column: cell.column }
    case 'columnRight':
      return { kind: 'insertColumn', column: cell.column + 1 }
    case 'deleteRow':
      return { kind: 'deleteRow', row: cell.row - 1 }
    case 'deleteColumn':
      return { kind: 'deleteColumn', column: cell.column }
    case 'alignLeft':
      return { kind: 'align', column: cell.column, alignment: 'left' }
    case 'alignCenter':
      return { kind: 'align', column: cell.column, alignment: 'center' }
    case 'alignRight':
      return { kind: 'align', column: cell.column, alignment: 'right' }
  }
}
