import { TerminalView } from "./components/terminal/TerminalView";

function App() {
  return (
    <div className="flex h-screen w-screen bg-[#1e1e2e]">
      <main className="flex-1 flex flex-col">
        <TerminalView />
      </main>
    </div>
  );
}

export default App;
