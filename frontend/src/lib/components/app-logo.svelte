<script lang="ts">
  import { APP_NAME } from '$lib/app';
  import AppLogoIcon from './app-logo-icon.svelte';
</script>

<AppLogoIcon class="size-8 rounded-md" />
<div class="ml-1 grid flex-1 text-left text-sm">
  <span class="truncate leading-tight font-semibold">{APP_NAME}</span>
</div>
