import { useState } from "react";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    // setGreetMsg(await invoke("greet", { name }));
    setGreetMsg(`Hello, ${name}! (Backend not connected yet)`);
  }

  return (
    <div className="container mx-auto p-4 text-center">
      <h1 className="text-3xl font-bold mb-4">Welcome to SquadSync</h1>

      <div className="flex flex-col items-center gap-4">
        <input
          id="greet-input"
          className="border p-2 rounded text-black"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button 
            type="button" 
            className="bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600"
            onClick={() => greet()}
        >
          Greet
        </button>
      </div>

      <p className="mt-4">{greetMsg}</p>
    </div>
  );
}

export default App;
