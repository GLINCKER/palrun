#!/usr/bin/env node

/**
 * Palrun npm package uninstaller
 *
 * Cleans up the downloaded binary.
 */

const fs = require('fs');
const path = require('path');
const os = require('os');

const PACKAGE_NAME = 'palrun';

function uninstall() {
  console.log(`Uninstalling ${PACKAGE_NAME}...`);

  const binDir = path.join(__dirname, '..', 'bin');
  const ext = os.platform() === 'win32' ? '.exe' : '';
  const binaryPath = path.join(binDir, `${PACKAGE_NAME}${ext}`);

  try {
    if (fs.existsSync(binaryPath)) {
      fs.unlinkSync(binaryPath);
      console.log(`Removed ${binaryPath}`);
    }

    // Clean up bin directory if empty
    if (fs.existsSync(binDir)) {
      const remaining = fs.readdirSync(binDir);
      if (remaining.length === 0) {
        fs.rmdirSync(binDir);
      }
    }

    console.log(`${PACKAGE_NAME} uninstalled successfully.`);
  } catch (err) {
    console.error(`Uninstall warning: ${err.message}`);
    // Don't fail the uninstall
  }
}

uninstall();
