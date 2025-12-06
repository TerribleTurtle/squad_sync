export interface AudioSyncResult {
  url: string;
  offsetMs: number;
  confidence: number;
}

// Declare global to extend the Window interface correctly
declare global {
  interface Window {
    webkitAudioContext: typeof AudioContext;
  }
}

/**
 * Align multiple video clips based on their audio tracks.
 * Uses Normalized Cross-Correlation to find the time offset between clips.
 *
 * @param clips List of video objects with 'url' property
 * @returns Map of URL -> Offset in MS (relative to the first clip)
 */
export async function alignClips(clips: { url: string }[]): Promise<Map<string, number>> {
  if (clips.length < 2) return new Map();

  const AudioContextClass = window.AudioContext || window.webkitAudioContext;
  const ctx = new AudioContextClass();
  const offsets = new Map<string, number>();

  const MIN_CONFIDENCE = 0.4; // Require 40% correlation match
  const SILENCE_THRESHOLD = 0.01; // Audio is considered silent if RMS < 0.01
  const MAX_ANALYSIS_DURATION_SEC = 30; // Only analyze first 30 seconds

  // Helper: Fetch and Decode Audio
  const getAudioBuffer = async (url: string): Promise<AudioBuffer> => {
    // Optimization: Fetch only the beginning of the file.
    // 192kbps = 24KB/s. 30s ~= 720KB.
    // Fetching 2MB is safe for >60s of high quality audio or >30s of lossless/wav.
    const response = await fetch(url, {
      headers: { Range: 'bytes=0-2000000' },
    });
    const arrayBuffer = await response.arrayBuffer();
    return await ctx.decodeAudioData(arrayBuffer);
  };

  try {
    // 1. Get Reference Buffer (First Clip)
    const refBuffer = await getAudioBuffer(clips[0].url);
    const refData = downsampleAndNormalize(refBuffer, 1000, MAX_ANALYSIS_DURATION_SEC);

    // Check for Reference Silence
    if (getRMS(refData) < SILENCE_THRESHOLD) {
      console.warn('Reference clip is silent, cannot perform Magic Sync.');
      return new Map();
    }

    offsets.set(clips[0].url, 0);

    // 2. Compare others to Reference
    for (let i = 1; i < clips.length; i++) {
      const targetUrl = clips[i].url;
      const targetBuffer = await getAudioBuffer(targetUrl);
      const targetData = downsampleAndNormalize(targetBuffer, 1000, MAX_ANALYSIS_DURATION_SEC);

      // Check for Target Silence
      if (getRMS(targetData) < SILENCE_THRESHOLD) {
        console.warn(`Clip ${i} is silent, keeping timestamp sync.`);
        offsets.set(targetUrl, 0);
        continue;
      }

      const { lag, confidence } = correlateNormalized(refData, targetData);

      if (confidence > MIN_CONFIDENCE) {
        console.log(`Synced ${targetUrl}: Offset=${lag}ms, Confidence=${confidence.toFixed(2)}`);
        offsets.set(targetUrl, lag);
      } else {
        console.warn(
          `Low confidence for ${targetUrl} (${confidence.toFixed(2)}). Fallback to timestamps.`
        );
        offsets.set(targetUrl, 0);
      }
    }
  } catch (e) {
    console.warn('Audio Sync Failed:', e);
  } finally {
    if (ctx.state !== 'closed') ctx.close().catch(() => {});
  }

  return offsets;
}

// Downsample and Normalize (Mean Subtraction)
function downsampleAndNormalize(
  buffer: AudioBuffer,
  targetRate: number,
  maxDurationSec: number = 30
): Float32Array {
  const inputData = buffer.getChannelData(0); // Use Left Channel
  const ratio = buffer.sampleRate / targetRate;

  // Calculate output length based on MAX duration or actual buffer duration
  const durationToProcess = Math.min(buffer.duration, maxDurationSec);
  const outputLength = Math.floor(durationToProcess * targetRate);

  const output = new Float32Array(outputLength);

  let sum = 0;

  for (let i = 0; i < outputLength; i++) {
    const start = Math.floor(i * ratio);
    const end = Math.floor((i + 1) * ratio);

    // Bounds check
    if (start >= inputData.length) break;

    let binSum = 0;
    const realEnd = Math.min(end, inputData.length);

    for (let j = start; j < realEnd; j++) {
      binSum += inputData[j]; // Sum
    }
    const val = binSum / (realEnd - start);
    output[i] = val;
    sum += val;
  }

  // Mean Subtraction (dc removal)
  const mean = sum / outputLength;
  for (let i = 0; i < outputLength; i++) {
    output[i] -= mean;
  }

  return output;
}

function getRMS(data: Float32Array): number {
  let sumSq = 0;
  for (let i = 0; i < data.length; i++) {
    sumSq += data[i] * data[i];
  }
  return Math.sqrt(sumSq / data.length);
}

// Normalized Cross-Correlation
function correlateNormalized(
  ref: Float32Array,
  target: Float32Array
): { lag: number; confidence: number } {
  const n = ref.length;
  const m = target.length;
  // Constrain search to +/- 2000ms
  const range = 2000;

  let maxCorr = -1;
  let bestLag = 0;

  for (let lag = -range; lag <= range; lag++) {
    // 1. Determine Overlap Region
    const start = Math.max(0, -lag);
    const end = Math.min(n, m - lag);

    // Safety check for minimal overlap
    if (end - start < 1000) continue;

    let sumProd = 0;
    let sumSqRef = 0;
    let sumSqTarget = 0;

    for (let i = start; i < end; i++) {
      const r = ref[i];
      const t = target[i + lag];
      sumProd += r * t;
      sumSqRef += r * r;
      sumSqTarget += t * t;
    }

    if (sumSqRef === 0 || sumSqTarget === 0) continue;

    // Pearson Correlation
    const correlation = sumProd / (Math.sqrt(sumSqRef) * Math.sqrt(sumSqTarget));

    if (correlation > maxCorr) {
      maxCorr = correlation;
      bestLag = lag;
    }
  }

  return { lag: bestLag, confidence: maxCorr };
}
