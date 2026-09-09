// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
  namespace App {
    // interface Error {}
    interface Locals {
      /** The appearance cookie's choice for this request (see $lib/appearance). */
      appearance: import('$lib/types').Appearance;
      /** The signed-in account, resolved by hooks.server.ts on every request. */
      user: import('$lib/types').User | null;
    }
    interface PageData {
      /** Who is signed in, on every page; null for a guest. */
      user: import('$lib/types').User | null;
      /** The breadcrumb trail a page declares for the app header. */
      breadcrumbs?: import('$lib/types').BreadcrumbItem[];
    }
    // interface PageState {}
    // interface Platform {}
  }
}

export {};
