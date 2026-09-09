<script lang="ts">
  import type { HTMLImgAttributes } from 'svelte/elements';
  import { APP_NAME } from '$lib/app';

  let { class: className, ...rest }: HTMLImgAttributes = $props();
</script>

<img src="/icon-192.png" alt={APP_NAME} class={className} {...rest} />
