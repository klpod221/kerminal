/**
 * Debounce function to limit the rate at which a function can fire.
 * @param func - The function to debounce.
 * @param wait - The number of milliseconds to wait before invoking the function.
 * @returns A debounced version of the input function.
 */
export function debounce<T extends (...args: any[]) => any>(
  func: T,
  wait: number,
): (...args: Parameters<T>) => void {
  let timeoutId: ReturnType<typeof setTimeout> | null = null;

  return (...args: Parameters<T>) => {
    if (timeoutId !== null) {
      clearTimeout(timeoutId);
    }

    timeoutId = setTimeout(() => func(...args), wait);
  };
}
