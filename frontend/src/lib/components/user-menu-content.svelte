<script lang="ts">
  // The account menu: who is signed in, settings, and logging out.
  import { LogOut, Settings } from '@lucide/svelte';
  import type { User } from '$lib/types';
  import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
  import UserInfo from './user-info.svelte';

  let { user }: { user: User } = $props();
</script>

<DropdownMenu.Label class="p-0 font-normal">
  <div class="flex items-center gap-2 px-1 py-1.5 text-left text-sm">
    <UserInfo {user} showEmail />
  </div>
</DropdownMenu.Label>
<DropdownMenu.Separator />
<DropdownMenu.Group>
  <DropdownMenu.Item>
    {#snippet child({ props })}
      <a {...props} href="/settings/profile">
        <Settings class="size-4" />
        Settings
      </a>
    {/snippet}
  </DropdownMenu.Item>
</DropdownMenu.Group>
<DropdownMenu.Separator />
<DropdownMenu.Item>
  {#snippet child({ props })}
    <!-- A plain form post: the API answers a browser with the redirect. -->
    <form method="post" action="/api/logout" class="contents">
      <button {...props} type="submit" class="{props.class} w-full" data-testid="logout-button">
        <LogOut class="size-4" />
        Log out
      </button>
    </form>
  {/snippet}
</DropdownMenu.Item>
