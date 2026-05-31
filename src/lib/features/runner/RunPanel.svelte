<script lang="ts">
  import Console, { type ConsoleApi } from "../../ui/Console.svelte";
  import { startRun, type RunEvent, type RunArg } from "../../ipc/api";

  // just runs whatever body it's handed (with the resolved param args) and streams the
  // result. it doesnt know or care whether that body is a saved script or something
  // half-typed in the editor.
  let { body, args }: { body: string; args: RunArg[] } = $props();

  let term: ConsoleApi | undefined = $state();
  let running = $state(false);

  async function run() {
    if (running || !term) return;
    running = true;
    term.clear();
    term.write("\x1b[90m> running...\x1b[0m\r\n");

    try {
      await startRun(body, args, (ev: RunEvent) => {
        if (ev.type === "stdout" || ev.type === "stderr") {
          term?.write(ev.chunk);
        } else if (ev.type === "exit") {
          const code = ev.code === null ? "" : `, code ${ev.code}`;
          term?.write(`\r\n\x1b[90m> ${ev.status}${code}\x1b[0m\r\n`);
          running = false;
        }
      });
    } catch (e) {
      term?.write(`\r\n\x1b[31m${String(e)}\x1b[0m\r\n`);
      running = false;
    }
  }
</script>

<div class="runner">
  <div class="bar">
    <button onclick={run} disabled={running || !body.trim()}>
      {running ? "Running..." : "Run"}
    </button>
  </div>
  <div class="output">
    <Console onReady={(api) => (term = api)} />
  </div>
</div>

<style>
  .runner {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
  }
  .bar {
    padding: 0.4rem 0.6rem;
    border-bottom: 1px solid #333;
  }
  button {
    padding: 0.4rem 1.2rem;
    cursor: pointer;
  }
  button:disabled {
    cursor: default;
    opacity: 0.6;
  }
  .output {
    flex: 1;
    min-height: 0;
  }
</style>
