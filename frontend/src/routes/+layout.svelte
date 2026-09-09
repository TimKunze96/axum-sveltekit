<script lang="ts">
  // The root frame every page shares: the theme, the saved appearance
  // applied on load, and the toaster. The sidebar shell lives in the
  // (app) group, the centered card in (guest); the welcome page stands
  // alone.
  import './layout.css';
  import type { Snippet } from 'svelte';
  import { initializeTheme } from '$lib/appearance.svelte';
  import AppNotifications from '$lib/components/app-notifications.svelte';
  import type { LayoutData } from './$types';

  let { data, children }: { data: LayoutData; children: Snippet } = $props();

  $effect(() => initializeTheme(data.appearance));
</script>

{@render children()}
<AppNotifications notification={data.notification} />
