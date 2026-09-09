<script lang="ts">
  // The account's initials, name and email in a menu row.
  import { getInitials } from '$lib/initials';
  import type { User } from '$lib/types';
  import * as Avatar from '$lib/components/ui/avatar';

  let { user, showEmail = false }: { user: User; showEmail?: boolean } = $props();
</script>

<Avatar.Root class="h-8 w-8 overflow-hidden rounded-lg">
  <Avatar.Fallback class="rounded-lg text-black dark:text-white">
    {getInitials(user.name)}
  </Avatar.Fallback>
</Avatar.Root>

<div class="grid flex-1 text-left text-sm leading-tight">
  <span class="truncate font-medium">{user.name}</span>
  {#if showEmail}
    <span class="truncate text-xs text-muted-foreground">{user.email}</span>
  {/if}
</div>
