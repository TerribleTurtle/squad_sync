import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Settings } from "./components/Settings";

function App() {
  const [status, setStatus] = useState("");
  const [showSettings, setShowSettings] = useState(false);

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
    <div className="container mx-auto p-4 text-center pb-20">
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

      <div className="p-4 bg-gray-100 rounded text-black mb-8">
        <p className="font-mono">{status}</p>
      </div>

      <button
        onClick={() => setShowSettings(!showSettings)}
        className="text-gray-500 hover:text-gray-700 underline mb-4"
      >
        {showSettings ? "Hide Settings" : "Show Settings"}
      </button>

      {showSettings && <Settings />}
    </div>
  );
}

export default App;
