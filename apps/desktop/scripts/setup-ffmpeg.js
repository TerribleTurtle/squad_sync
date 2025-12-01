import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import https from 'https';
import { execSync } from 'child_process';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const dest = path.resolve(__dirname, '../src-tauri/bin');

// Ensure destination exists
if (!fs.existsSync(dest)) {
  fs.mkdirSync(dest, { recursive: true });
}

const platform = process.platform;
const arch = process.arch;

console.log(`Detected platform: ${platform} (${arch})`);

// Configuration for "Full" builds (GPL)
// Using BtbN for Windows/Linux and Evermeet/osxexperts for Mac (if needed, but focusing on Windows for now)
const DOWNLOADS = {
  win32: {
    url: 'https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip',
    filename: 'ffmpeg.zip',
    extractCmd: 'powershell -Command "Expand-Archive -Path ffmpeg.zip -DestinationPath . -Force"',
    binaries: [
      {
        src: 'ffmpeg-master-latest-win64-gpl/bin/ffmpeg.exe',
        dest: 'ffmpeg-x86_64-pc-windows-msvc.exe',
      },
      {
        src: 'ffmpeg-master-latest-win64-gpl/bin/ffprobe.exe',
        dest: 'ffprobe-x86_64-pc-windows-msvc.exe',
      },
    ],
  },
  // TODO: Add Mac/Linux support if needed. For now, this fixes the user's Windows issue.
};

async function downloadFile(url, destPath) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(destPath);
    https
      .get(url, (response) => {
        if (response.statusCode === 302 || response.statusCode === 301) {
          downloadFile(response.headers.location, destPath).then(resolve).catch(reject);
          return;
        }
        response.pipe(file);
        file.on('finish', () => {
          file.close(resolve);
        });
      })
      .on('error', (err) => {
        fs.unlink(destPath, () => reject(err));
      });
  });
}

async function main() {
  if (platform !== 'win32') {
    console.error('This script currently only supports Windows auto-download of Full builds.');
    console.error('Please manually install FFmpeg for your platform.');
    return;
  }

  const config = DOWNLOADS[platform];
  const zipPath = path.join(dest, config.filename);

  // Move and Rename
  let allBinariesExist = true;
  for (const bin of config.binaries) {
    const destPath = path.join(dest, bin.dest);
    if (!fs.existsSync(destPath)) {
      allBinariesExist = false;
      break;
    }
  }

  if (allBinariesExist) {
    console.log('FFmpeg binaries already exist. Skipping download.');
    return;
  }

  console.log(`Downloading Full FFmpeg build from ${config.url}...`);
  try {
    await downloadFile(config.url, zipPath);
    console.log('Download complete. Extracting...');

    // Extract
    execSync(config.extractCmd, { cwd: dest, stdio: 'inherit' });
    console.log('Extraction complete.');

    for (const bin of config.binaries) {
      const srcPath = path.join(dest, bin.src);
      const destPath = path.join(dest, bin.dest);

      if (fs.existsSync(srcPath)) {
        fs.copyFileSync(srcPath, destPath);
        console.log(`Installed: ${path.basename(destPath)}`);
      } else {
        console.error(`Error: Could not find extracted binary at ${srcPath}`);
      }
    }

    // Cleanup
    console.log('Cleaning up...');
    if (fs.existsSync(zipPath)) fs.unlinkSync(zipPath);
    // Cleanup extracted folder (assuming it matches the zip content structure)
    const extractedFolder = path.join(dest, 'ffmpeg-master-latest-win64-gpl');
    if (fs.existsSync(extractedFolder))
      fs.rmSync(extractedFolder, { recursive: true, force: true });

    console.log('FFmpeg setup complete (Full Build).');
  } catch (error) {
    console.error('Failed to setup FFmpeg:', error);
    process.exit(1);
  }
}

main();
