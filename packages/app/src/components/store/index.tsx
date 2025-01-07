import { createContext, useContext, useMemo } from "react";
import { create } from "zustand";
import { immer } from "zustand/middleware/immer";

type StoreState = {
  devices: any[];
};

const makeStore = () =>
  create(
    immer<StoreState>((set, get) => ({
      devices: [],
    }))
  );

const Store = createContext<ReturnType<typeof makeStore>>(makeStore());

const StoreProvider = ({ children }: { children: React.ReactNode }) => {
  const store = useMemo(() => makeStore(), []);
  return <Store.Provider value={store}>{children}</Store.Provider>;
};

const useStore = () => useContext(Store);

export { StoreProvider, useStore };
