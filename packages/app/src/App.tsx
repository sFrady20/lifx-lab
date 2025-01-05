import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { Button } from "@/components/ui/button";

function App() {
  return (
    <main className="">
      <div className="text-xl font-bold">Lifx Lab</div>
      <Button
        onClick={async () => {
          await invoke("lights_on");
        }}
      >
        <i className="icon-[lucide--lightbulb]" />
        <div>All Lights On</div>
      </Button>
    </main>
  );
}

export default App;
