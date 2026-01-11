#!/usr/bin/env node

/**
 * Palrun npm package installer
 *
 * Downloads the appropriate pre-built binary for the current platform.
 */

const { execSync } = require('child_process');
const fs = require('fs');
const https = require('https');
const path = require('path');
const os = require('os');
const zlib = require('zlib');

// Configuration
const PACKAGE_NAME = 'palrun';
const REPO = 'GLINCKER/palrun';
const VERSION = require('../package.json').version;

// Platform mappings
const PLATFORMS = {
  darwin: {
    x64: 'darwin-x86_64',
    arm64: 'darwin-aarch64',
  },
  linux: {
    x64: 'linux-x86_64',
    arm64: 'linux-aarch64',
  },
  win32: {
    x64: 'windows-x86_64',
  },
};

/**
 * Get the platform-specific binary name.
 */
function getPlatformBinary() {
  const platform = os.platform();
  const arch = os.arch();

  const platformMap = PLATFORMS[platform];
  if (!platformMap) {
    throw new Error(`Unsupported platform: ${platform}`);
  }

  const suffix = platformMap[arch];
  if (!suffix) {
    throw new Error(`Unsupported architecture: ${arch} on ${platform}`);
  }

  const ext = platform === 'win32' ? '.exe' : '';
  return {
    suffix,
    filename: `${PACKAGE_NAME}${ext}`,
    archive: platform === 'win32' ? 'zip' : 'tar.gz',
  };
}

/**
 * Download a file from URL.
 */
function download(url, dest) {
  return new Promise((resolve, reject) => {
    console.log(`Downloading from ${url}...`);

    const request = https.get(url, (response) => {
      // Handle redirects
      if (response.statusCode >= 300 && response.statusCode < 400 && response.headers.location) {
        return download(response.headers.location, dest).then(resolve).catch(reject);
      }

      if (response.statusCode !== 200) {
        reject(new Error(`Download failed with status ${response.statusCode}`));
        return;
      }

      const file = fs.createWriteStream(dest);
      response.pipe(file);

      file.on('finish', () => {
        file.close();
        resolve();
      });

      file.on('error', (err) => {
        fs.unlink(dest, () => {});
        reject(err);
      });
    });

    request.on('error', reject);
    request.setTimeout(60000, () => {
      request.destroy();
      reject(new Error('Download timed out'));
    });
  });
}

/**
 * Extract tar.gz archive.
 */
function extractTarGz(archive, dest) {
  return new Promise((resolve, reject) => {
    try {
      // Use tar command if available
      execSync(`tar -xzf "${archive}" -C "${dest}"`, { stdio: 'pipe' });
      resolve();
    } catch (err) {
      reject(new Error(`Failed to extract archive: ${err.message}`));
    }
  });
}

/**
 * Extract zip archive.
 */
function extractZip(archive, dest) {
  return new Promise((resolve, reject) => {
    try {
      // Use unzip command if available, or PowerShell on Windows
      if (os.platform() === 'win32') {
        execSync(`powershell -Command "Expand-Archive -Path '${archive}' -DestinationPath '${dest}' -Force"`, { stdio: 'pipe' });
      } else {
        execSync(`unzip -o "${archive}" -d "${dest}"`, { stdio: 'pipe' });
      }
      resolve();
    } catch (err) {
      reject(new Error(`Failed to extract archive: ${err.message}`));
    }
  });
}

/**
 * Main installation function.
 */
async function install() {
  console.log(`Installing ${PACKAGE_NAME} v${VERSION}...`);

  try {
    const { suffix, filename, archive } = getPlatformBinary();
    const binDir = path.join(__dirname, '..', 'bin');
    const tmpDir = os.tmpdir();

    // Ensure bin directory exists
    if (!fs.existsSync(binDir)) {
      fs.mkdirSync(binDir, { recursive: true });
    }

    // Build download URL
    const archiveName = `${PACKAGE_NAME}-${suffix}.${archive}`;
    const url = `https://github.com/${REPO}/releases/download/v${VERSION}/${archiveName}`;
    const archivePath = path.join(tmpDir, archiveName);
    const extractDir = path.join(tmpDir, `${PACKAGE_NAME}-extract`);

    // Download the archive
    await download(url, archivePath);

    // Create extraction directory
    if (fs.existsSync(extractDir)) {
      fs.rmSync(extractDir, { recursive: true });
    }
    fs.mkdirSync(extractDir, { recursive: true });

    // Extract the archive
    console.log('Extracting...');
    if (archive === 'zip') {
      await extractZip(archivePath, extractDir);
    } else {
      await extractTarGz(archivePath, extractDir);
    }

    // Find and move the binary
    const binaryPath = path.join(binDir, filename);
    const extractedBinary = findBinary(extractDir, filename);

    if (!extractedBinary) {
      throw new Error(`Binary not found in archive`);
    }

    fs.copyFileSync(extractedBinary, binaryPath);

    // Make executable on Unix
    if (os.platform() !== 'win32') {
      fs.chmodSync(binaryPath, 0o755);
    }

    // Cleanup
    fs.rmSync(archivePath, { force: true });
    fs.rmSync(extractDir, { recursive: true, force: true });

    console.log(`${PACKAGE_NAME} v${VERSION} installed successfully!`);
    console.log(`Binary location: ${binaryPath}`);
    console.log('');
    console.log('Run `palrun` or `pal` to get started.');

  } catch (err) {
    console.error(`Installation failed: ${err.message}`);
    console.error('');
    console.error('You can install manually:');
    console.error('  cargo install palrun');
    console.error('  # or download from https://github.com/GLINCKER/palrun/releases');
    process.exit(1);
  }
}

/**
 * Find binary in extracted directory.
 */
function findBinary(dir, filename) {
  const entries = fs.readdirSync(dir, { withFileTypes: true });

  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);

    if (entry.isDirectory()) {
      const found = findBinary(fullPath, filename);
      if (found) return found;
    } else if (entry.name === filename || entry.name === 'palrun' || entry.name === 'palrun.exe') {
      return fullPath;
    }
  }

  return null;
}

// Run installation
install();
