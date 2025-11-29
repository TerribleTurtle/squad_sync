import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

function App() {
  const [status, setStatus] = useState("");

  async function startRecording() {
    try {
      await invoke("start_recording");
      setStatus("Recording started...");
    } catch (e) {
      setStatus(`Error starting: ${e}`);
    }
  }

  async function stopRecording() {
    try {
      await invoke("stop_recording");
      setStatus("Recording stopped.");
    } catch (e) {
      setStatus(`Error stopping: ${e}`);
    }
  }

  return (
    <div className="container mx-auto p-4 text-center">
      <h1 className="text-3xl font-bold mb-8">SquadSync Recorder</h1>

      <div className="flex justify-center gap-4 mb-8">
        <button 
            type="button" 
            className="bg-red-500 text-white px-6 py-3 rounded-lg hover:bg-red-600 font-bold"
            onClick={startRecording}
        >
          Start Recording
        </button>
        
        <button 
            type="button" 
            className="bg-gray-700 text-white px-6 py-3 rounded-lg hover:bg-gray-800 font-bold"
            onClick={stopRecording}
        >
          Stop Recording
        </button>
      </div>

      <div className="p-4 bg-gray-100 rounded text-black">
        <p className="font-mono">{status}</p>
      </div>
    </div>
  );
}

export default App;
