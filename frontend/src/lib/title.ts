import { APP_NAME } from '$lib/app';

/** `{title} - {app}`, or the app name alone. */
export function pageTitle(title?: string): string {
  return title ? `${title} - ${APP_NAME}` : APP_NAME;
}
