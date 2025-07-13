export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0.00 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
}

/**
 * Converts a duration in seconds to a human-readable string format.
 *
 * @param seconds - Duration in seconds
 * @returns Formatted string in the format "Xh00m00s", "Xm00s", or "Xs"
 *
 * @example
 * etaToHumanReadable(3661) // "1h01m01s"
 * etaToHumanReadable(61)   // "1m01s"
 * etaToHumanReadable(30)   // "30s"
 * etaToHumanReadable(0)    // "--"
 */
export function etaToHumanReadable(seconds: number): string {
  if (seconds <= 0 || seconds > 86400) {
    return '--';
  }

  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);

  if (hours > 0) {
    return `${hours}h${minutes.toString().padStart(2, '0')}m${secs.toString().padStart(2, '0')}s`;
  } else if (minutes > 0) {
    return `${minutes}m${secs.toString().padStart(2, '0')}s`;
  } else {
    return `${secs}s`;
  }
}
