import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Settings } from "./components/Settings";

function App() {
  const [status, setStatus] = useState("");
  const [isReplayActive, setIsReplayActive] = useState(false);
  const [isBuffering, setIsBuffering] = useState(false);
  const [showSettings, setShowSettings] = useState(false);

  async function enableReplay() {
    try {
      setIsBuffering(true);
      setStatus("Buffering...");
      await invoke("enable_replay");
      
      // Simulate buffering delay for UX
      setTimeout(() => {
        setIsBuffering(false);
        setIsReplayActive(true);
        setStatus("Replay Buffer Active - Ready to Clip");
      }, 2000);
    } catch (e) {
      setIsBuffering(false);
      setStatus(`Error enabling replay: ${e}`);
    }
  }

  async function disableReplay() {
    try {
      await invoke("disable_replay");
      setIsReplayActive(false);
      setStatus("Replay Buffer Disabled");
    } catch (e) {
      setStatus(`Error disabling: ${e}`);
    }
  }

  async function saveReplay() {
    try {
      setStatus("Saving Clip...");
      const path = await invoke("save_replay");
      setStatus(`Clip Saved! (${path})`);
      setTimeout(() => setStatus("Replay Buffer Active - Ready to Clip"), 3000);
    } catch (e) {
      setStatus(`Error saving clip: ${e}`);
    }
  }

  return (
    <div className="container mx-auto p-4 text-center pb-20">
      <h1 className="text-3xl font-bold mb-8">SquadSync Recorder</h1>

      <div className="flex flex-col items-center gap-4 mb-8">
        {!isReplayActive ? (
            <button 
                type="button" 
                className={`px-8 py-4 rounded-lg font-bold text-xl transition-all ${
                    isBuffering ? "bg-yellow-500 cursor-wait" : "bg-green-600 hover:bg-green-700 text-white"
                }`}
                onClick={enableReplay}
                disabled={isBuffering}
            >
              {isBuffering ? "Buffering..." : "Enable Replay Buffer"}
            </button>
        ) : (
            <div className="flex gap-4">
                <button 
                    type="button" 
                    className="bg-blue-600 text-white px-8 py-4 rounded-lg hover:bg-blue-700 font-bold text-xl animate-pulse"
                    onClick={saveReplay}
                >
                  Save Instant Replay
                </button>
                
                <button 
                    type="button" 
                    className="bg-red-600 text-white px-6 py-4 rounded-lg hover:bg-red-700 font-bold"
                    onClick={disableReplay}
                >
                  Disable
                </button>
            </div>
        )}
      </div>

      <div className="p-4 bg-gray-100 rounded text-black mb-8 min-h-[60px] flex items-center justify-center">
        <p className="font-mono text-lg">{status}</p>
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
