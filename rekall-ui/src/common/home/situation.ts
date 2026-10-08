/**
 * The home screen's headline: what is working and what is waiting on you, in one sentence of two
 * halves. `urgent` is true when the second half asks for something, which is what lights it amber.
 */
export interface Situation {
  readonly working: string
  readonly waiting: string
  readonly urgent: boolean
}

export function situationOf(liveSessions: number, waitingForReview: number): Situation {
  const working =
    liveSessions === 0
      ? 'Nothing running,'
      : `${liveSessions} ${liveSessions === 1 ? 'session' : 'sessions'} working,`
  const urgent = waitingForReview > 0
  const waiting = urgent ? `${waitingForReview} waiting for your review.` : 'nothing waiting for you.'
  return { working, waiting, urgent }
}
