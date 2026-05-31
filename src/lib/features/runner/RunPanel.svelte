<script lang="ts">
  import Console, { type ConsoleApi } from "../../ui/Console.svelte";
  import { startRun, type RunEvent } from "../../ipc/api";

  // a little starter snippet so the demo does something visibly streaming (the sleep
  // makes the lines arrive one at a time instead of all at once).
  let command = $state(
    `Write-Output "hello from EasiShell"\n1..5 | ForEach-Object { "line $_"; Start-Sleep -Milliseconds 250 }`,
  );
  let running = $state(false);
  let term: ConsoleApi | undefined = $state();

  async function run() {
    if (running || !term) return;
    running = true;
    term.clear();
    term.write("\x1b[90m> running...\x1b[0m\r\n");

    try {
      await startRun(command, (ev: RunEvent) => {
        if (ev.type === "stdout" || ev.type === "stderr") {
          term?.write(ev.chunk);
        } else if (ev.type === "exit") {
          const code = ev.code === null ? "" : `, code ${ev.code}`;
          term?.write(`\r\n\x1b[90m> ${ev.status}${code}\x1b[0m\r\n`);
          running = false;
        }
      });
    } catch (e) {
      // start_run itself failed (couldnt spawn, couldnt write the temp file, etc.)
      term?.write(`\r\n\x1b[31m${String(e)}\x1b[0m\r\n`);
      running = false;
    }
  }
</script>

<section class="panel">
  <header>
    <h1>EasiShell</h1>
    <button onclick={run} disabled={running}>{running ? "Running..." : "Run"}</button>
  </header>

  <textarea
    bind:value={command}
    spellcheck="false"
    placeholder="Type some PowerShell, then hit Run"
  ></textarea>

  <div class="output">
    <Console onReady={(api) => (term = api)} />
  </div>
</section>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    font-family: system-ui, sans-serif;
  }
  header {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.6rem 1rem;
    border-bottom: 1px solid #333;
  }
  h1 {
    margin: 0;
    font-size: 1.1rem;
    flex: 1;
  }
  button {
    padding: 0.4rem 1.1rem;
    font-size: 0.95rem;
    cursor: pointer;
  }
  button:disabled {
    cursor: default;
    opacity: 0.6;
  }
  textarea {
    font-family: Consolas, "Cascadia Mono", monospace;
    font-size: 0.9rem;
    height: 7rem;
    resize: vertical;
    border: none;
    border-bottom: 1px solid #333;
    padding: 0.6rem 1rem;
    outline: none;
  }
  .output {
    flex: 1;
    min-height: 0; /* lets the console shrink/scroll instead of pushing the layout */
  }
</style>
