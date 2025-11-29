import ffbinaries from 'ffbinaries';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const platform = ffbinaries.detectPlatform();
const dest = path.resolve(__dirname, '../src-tauri/bin');

if (!fs.existsSync(dest)) {
  fs.mkdirSync(dest, { recursive: true });
}

console.log(`Detected platform: ${platform}`);
console.log(`Downloading FFmpeg to ${dest}...`);

ffbinaries.downloadBinaries(['ffmpeg'], { destination: dest }, function () {
  console.log('Downloaded FFmpeg.');

  // Rename to match Tauri's expected format (with target triple)
  // Current assumption: Windows x64
  if (platform.startsWith('win')) {
    const source = path.join(dest, 'ffmpeg.exe');
    const target = path.join(dest, 'ffmpeg-x86_64-pc-windows-msvc.exe');
    
    if (fs.existsSync(source)) {
      fs.renameSync(source, target);
      console.log(`Renamed to ${path.basename(target)}`);
    }
  }
  
  console.log('FFmpeg setup complete.');
});
