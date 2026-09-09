<script lang="ts">
  // The public landing page: the pitch and the way in.
  import { ArrowRight } from '@lucide/svelte';
  import { APP_NAME } from '$lib/app';
  import { HOME_PATH, LOGIN_PATH, REGISTER_PATH } from '$lib/login';
  import { buttonVariants } from '$lib/components/ui/button';

  let { signedIn }: { signedIn: boolean } = $props();
</script>

<div class="flex min-h-screen flex-col items-center justify-center bg-background px-6 text-center">
  <main class="flex max-w-xl flex-col items-center">
    <img src="/icon-192.png" alt="" class="size-16 rounded-2xl shadow-sm" />
    <h1 class="mt-6 text-5xl font-semibold tracking-tight">{APP_NAME}</h1>
    <p class="mt-4 text-lg text-muted-foreground">
      An Axum API and a SvelteKit frontend on one origin, with accounts, sessions and settings
      already in place. Replace this page with your product.
    </p>

    <div class="mt-10 flex flex-wrap items-center justify-center gap-3">
      {#if signedIn}
        <a href={HOME_PATH} class={buttonVariants({ size: 'lg' })} data-testid="open-dashboard">
          Open the dashboard
          <ArrowRight class="size-4" />
        </a>
      {:else}
        <a href={LOGIN_PATH} class={buttonVariants({ size: 'lg' })} data-testid="login"> Log in </a>
        <a
          href={REGISTER_PATH}
          class={buttonVariants({ size: 'lg', variant: 'outline' })}
          data-testid="register"
        >
          Create an account
        </a>
      {/if}
    </div>
  </main>
</div>
