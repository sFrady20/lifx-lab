import "./App.css";
import { StoreProvider } from "./components/store";
import { HomePage } from "./pages/home";

function App() {
  return (
    <StoreProvider>
      <HomePage />
    </StoreProvider>
  );
}

export default App;
