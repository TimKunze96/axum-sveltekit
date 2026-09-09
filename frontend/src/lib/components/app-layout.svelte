<script lang="ts">
  // The signed-in shell: tooltips, the sidebar, the inset with its header,
  // and the page content.
  import type { Snippet } from 'svelte';
  import type { BreadcrumbItem, User } from '$lib/types';
  import * as Sidebar from '$lib/components/ui/sidebar';
  import * as Tooltip from '$lib/components/ui/tooltip';
  import AppSidebar from './app-sidebar.svelte';
  import AppSidebarHeader from './app-sidebar-header.svelte';

  let {
    user,
    sidebarOpen = true,
    breadcrumbs = [],
    children,
  }: {
    user: User;
    sidebarOpen?: boolean;
    breadcrumbs?: BreadcrumbItem[];
    children: Snippet;
  } = $props();
</script>

<Tooltip.Provider delayDuration={300}>
  <Sidebar.Provider open={sidebarOpen}>
    <AppSidebar {user} />
    <Sidebar.Inset class="overflow-x-hidden">
      <AppSidebarHeader {breadcrumbs} />
      {@render children()}
    </Sidebar.Inset>
  </Sidebar.Provider>
</Tooltip.Provider>
