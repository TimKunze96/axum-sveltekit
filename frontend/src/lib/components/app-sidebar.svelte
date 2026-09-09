<script lang="ts">
  // The sidebar: the logo, the platform links and the account menu. Add
  // a feature's pages to `platformNavItems`.
  import { LayoutGrid } from '@lucide/svelte';
  import { HOME_PATH } from '$lib/login';
  import type { NavItem, User } from '$lib/types';
  import * as Sidebar from '$lib/components/ui/sidebar';
  import AppLogo from './app-logo.svelte';
  import NavMain from './nav-main.svelte';
  import NavUser from './nav-user.svelte';

  let { user }: { user: User } = $props();

  const platformNavItems: NavItem[] = [{ title: 'Dashboard', href: HOME_PATH, icon: LayoutGrid }];
</script>

<Sidebar.Root collapsible="icon" variant="inset">
  <Sidebar.Header>
    <Sidebar.Menu>
      <Sidebar.MenuItem>
        <Sidebar.MenuButton size="lg">
          {#snippet child({ props })}
            <a {...props} href={HOME_PATH}>
              <AppLogo />
            </a>
          {/snippet}
        </Sidebar.MenuButton>
      </Sidebar.MenuItem>
    </Sidebar.Menu>
  </Sidebar.Header>

  <Sidebar.Content>
    <NavMain items={platformNavItems} />
  </Sidebar.Content>

  <Sidebar.Footer>
    <NavUser {user} />
  </Sidebar.Footer>
</Sidebar.Root>
