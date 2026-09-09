// Which navigation entry is the current page.

/** The path of a link, absolute URLs reduced to their path. */
function pathOf(href: string): string | null {
  if (!href.startsWith('http')) return href;
  try {
    return new URL(href).pathname;
  } catch {
    return null;
  }
}

export function isCurrentUrl(href: string, currentPath: string, startsWith = false): boolean {
  const path = pathOf(href);
  if (path === null) return false;
  return startsWith ? currentPath.startsWith(path) : path === currentPath;
}

export function isCurrentOrParentUrl(href: string, currentPath: string): boolean {
  return isCurrentUrl(href, currentPath, true);
}
