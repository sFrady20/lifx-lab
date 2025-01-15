import { useStore } from "@/components/store";
import { Button } from "@/components/ui/button";
import { invoke } from "@tauri-apps/api/core";
import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";

export const HomePage = function () {
  const store = useStore();

  const devices = store((x) => x.devices);

  useEffect(() => {
    const unlisten = listen("device_discovered", (event) => {
      console.log(`Device discovered`, event);
    });
    return () => {
      unlisten.then((x) => x());
    };
  }, []);

  return (
    <main className="flex flex-col gap-2">
      <div className="text-xl font-bold">Lifx Lab</div>
      {/* <Button
        onClick={async () => {
          const results = await invoke("discover_lights");
          console.log(results);
        }}
      >
        <i className="icon-[lucide--search]" />
        <div>Discover Lights</div>
      </Button> */}
      <div>
        {devices.map((device) => (
          <div>{device.ip}</div>
        ))}
      </div>
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
          await invoke("lights_off");
        }}
      >
        <i className="icon-[lucide--lightbulb-off]" />
        <div>All Lights Off</div>
      </Button>
      <Button
        onClick={async () => {
          await invoke("lights_set_color", {
            h: Math.round(Math.random() * 65535),
            s: Math.round(65535 * 0.5),
            b: Math.round(65535 * 0.5),
          });
        }}
      >
        <i className="icon-[lucide--lightbulb-off]" />
        <div>All Lights Random</div>
      </Button>
    </main>
  );
};
