// The wire types come from the Rust crate: ts-rs writes `./api/types`
// on `cargo test` (never edit those files). This module re-exports them
// and adds the UI-only types.
import type { Component } from 'svelte';

export type * from '$lib/api';

export type Appearance = 'light' | 'dark' | 'system';
export type ResolvedAppearance = 'light' | 'dark';

/** A lucide icon component. */
export type Icon = Component<{ class?: string }>;

export interface NavItem {
  title: string;
  href: string;
  icon?: Icon;
}

export interface BreadcrumbItem {
  title: string;
  href: string;
}
