/*
 * The dialogs open right now, oldest first. Every open dialog listens on `window`, so without
 * this a confirm opened over a panel would hand Escape to both and close the two at once. Only the
 * dialog on top answers the keyboard.
 */
const stack: symbol[] = []

export function enterDialog(): symbol {
  const token = Symbol('dialog')
  stack.push(token)
  return token
}

export function leaveDialog(token: symbol): void {
  const index = stack.indexOf(token)
  if (index !== -1) stack.splice(index, 1)
}

export function isTopDialog(token: symbol): boolean {
  return stack[stack.length - 1] === token
}
