import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { Button } from "@/components/ui/button";

function App() {
  return (
    <main className="flex flex-col gap-2">
      <div className="text-xl font-bold">Lifx Lab</div>
      <Button
        onClick={async () => {
          await invoke("lights_on");
        }}
      >
        <i className="icon-[lucide--lightbulb]" />
        <div>All Lights On</div>
      </Button>
      <Button
        onClick={async () => {
          await invoke("discover_lights");
        }}
      >
        <i className="icon-[lucide--search]" />
        <div>Discover Lights</div>
      </Button>
    </main>
  );
}

export default App;
