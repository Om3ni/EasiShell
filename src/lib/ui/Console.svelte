<script module lang="ts">
  // the little imperative handle the console hands back once it's mounted. kept tiny on
  // purpose — anything that needs to push text at the terminal goes through this rather
  // than reaching for the xterm instance directly.
  export interface ConsoleApi {
    write(chunk: string): void;
    clear(): void;
  }
</script>

<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Terminal } from "@xterm/xterm";
  import { FitAddon } from "@xterm/addon-fit";
  import "@xterm/xterm/css/xterm.css";

  let { onReady }: { onReady?: (api: ConsoleApi) => void } = $props();

  let host: HTMLDivElement;
  let term: Terminal | undefined;
  let fit: FitAddon | undefined;
  let resizeObs: ResizeObserver | undefined;

  onMount(() => {
    term = new Terminal({
      // convertEol stays off — PowerShell already sends \r\n, and we want the raw
      // control chars (carriage returns, ANSI) passed through untouched.
      convertEol: false,
      fontFamily: "Consolas, 'Cascadia Mono', monospace",
      fontSize: 13,
      scrollback: 10000,
      theme: { background: "#1e1e1e", foreground: "#e6e6e6" },
    });
    fit = new FitAddon();
    term.loadAddon(fit);
    term.open(host);
    fit.fit();

    // refit whenever the pane changes size. fit() can throw if the element is mid-layout
    // (0px), so its wrapped — not worth crashing over a transient resize.
    resizeObs = new ResizeObserver(() => {
      try {
        fit?.fit();
      } catch {
        /* element not measurable yet */
      }
    });
    resizeObs.observe(host);

    onReady?.({
      write: (chunk) => term?.write(chunk),
      clear: () => term?.clear(),
    });
  });

  onDestroy(() => {
    resizeObs?.disconnect();
    term?.dispose();
  });
</script>

<div class="console" bind:this={host}></div>

<style>
  .console {
    width: 100%;
    height: 100%;
    background: #1e1e1e;
  }
  /* xterm injects its own element; make it actually fill the box */
  .console :global(.xterm) {
    height: 100%;
    padding: 8px;
  }
</style>
