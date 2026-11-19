/** All browser writers of shared answer history use this lock. */
export const PREPARATION_LOCK = 'exam-preparation:history';
export async function withPreparationLock<T>(action: () => T | Promise<T>): Promise<T> {
  if (typeof navigator !== 'undefined' && navigator.locks) return navigator.locks.request(PREPARATION_LOCK, action);
  return action();
}
