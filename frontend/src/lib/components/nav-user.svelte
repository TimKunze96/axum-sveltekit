<script lang="ts">
  // The sidebar footer row opening the account menu.
  import { ChevronsUpDown } from '@lucide/svelte';
  import type { User } from '$lib/types';
  import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
  import * as Sidebar from '$lib/components/ui/sidebar';
  import UserInfo from './user-info.svelte';
  import UserMenuContent from './user-menu-content.svelte';

  let { user }: { user: User } = $props();

  const sidebar = Sidebar.useSidebar();
</script>

<Sidebar.Menu>
  <Sidebar.MenuItem>
    <DropdownMenu.Root>
      <DropdownMenu.Trigger>
        {#snippet child({ props })}
          <Sidebar.MenuButton
            {...props}
            size="lg"
            class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
            data-testid="sidebar-menu-button"
          >
            <UserInfo {user} />
            <ChevronsUpDown class="ml-auto size-4" />
          </Sidebar.MenuButton>
        {/snippet}
      </DropdownMenu.Trigger>
      <DropdownMenu.Content
        class="w-(--bits-dropdown-menu-anchor-width) min-w-56 rounded-lg"
        side={sidebar.isMobile ? 'bottom' : sidebar.state === 'collapsed' ? 'left' : 'bottom'}
        align="end"
        sideOffset={4}
      >
        <UserMenuContent {user} />
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  </Sidebar.MenuItem>
</Sidebar.Menu>
