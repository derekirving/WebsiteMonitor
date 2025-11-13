const { app, BrowserWindow, Tray, Menu, Notification, nativeImage } = require('electron');
const path = require('path');
const https = require('https');
const http = require('http');

let tray = null;
let mainWindow = null;
let monitorInterval = null;

// Configuration
const CONFIG = {
  url: 'https://www.google.com', // Change this to your website
  checkInterval: 60000, // Check every 60 seconds (in milliseconds)
  timeout: 10000 // Request timeout in milliseconds
};

let lastStatus = null;
let consecutiveFailures = 0;

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 400,
    height: 300,
    show: false,
    webPreferences: {
      nodeIntegration: true,
      contextIsolation: false
    }
  });

  mainWindow.loadFile('index.html');

  mainWindow.on('close', (event) => {
    if (!app.isQuitting) {
      event.preventDefault();
      mainWindow.hide();
    }
    return false;
  });
}

function createTrayIcon() {
  // Create a simple icon (you should replace this with actual icon files)
  const icon = nativeImage.createFromDataURL(
    'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAYAAABzenr0AAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAAJcEhZcwAADsMAAA7DAcdvqGQAAAETSURBVFhH7ZbBDcIwDEVTrgzABozABqzABmzABozACIzACIzQDWADOqWqVKlJ7aR1DlzgS5Eq59l/+Y0pY/6bfQJGYAqm4Aru4AEe4QU+Icd+C47gBC7gCt7hA3YQTkQD2YAz8AK/QRvZAH/gBfZg5AROoI/oAI/QhneAx2jDO8BjtOEd4DHa8A7wGG14B3iMNrwDPEYb3gEeow3vAI/RhneAx2jDO8BjtOEd4DHa8A7wGG14B3iMNrwDPEYb3gEeow3vAI/RhneAx2jDO8BjtOEd4DHa8A7wGG14B3iMNrwDPEYb3gEeow3vAI/RhneAx2jDO8BjtOEd4DHa8A7wGG14B3iMNrwDPEYb3gEeow3vAI/RhneAx2jDO8BjtJGNf2DMP2CW/QJhIJ8Vjvv1nAAAAABJRU5ErkJggg=='
  );
  
  tray = new Tray(icon);
  updateTrayMenu('Initializing...');
  
  tray.setToolTip('Website Monitor');
  
  tray.on('click', () => {
    mainWindow.show();
  });
}

function updateTrayMenu(status) {
  const contextMenu = Menu.buildFromTemplate([
    { label: `Status: ${status}`, enabled: false },
    { label: `Monitoring: ${CONFIG.url}`, enabled: false },
    { type: 'separator' },
    { label: 'Check Now', click: () => checkWebsite() },
    { label: 'Show Window', click: () => mainWindow.show() },
    { type: 'separator' },
    { label: 'Quit', click: () => {
      app.isQuitting = true;
      app.quit();
    }}
  ]);
  
  tray.setContextMenu(contextMenu);
}

function checkWebsite() {
  const urlObj = new URL(CONFIG.url);
  const protocol = urlObj.protocol === 'https:' ? https : http;
  
  const req = protocol.get(CONFIG.url, { timeout: CONFIG.timeout }, (res) => {
    const isSuccess = res.statusCode >= 200 && res.statusCode < 400;
    
    if (isSuccess) {
      consecutiveFailures = 0;
      updateTrayMenu('✓ Online');
      
      // Notify if recovering from failure
      if (lastStatus === 'down') {
        showNotification('Website is back online!', `${CONFIG.url} is now accessible.`);
      }
      
      lastStatus = 'up';
    } else {
      handleFailure(`HTTP ${res.statusCode}`);
    }
  });
  
  req.on('error', (err) => {
    handleFailure(err.message);
  });
  
  req.on('timeout', () => {
    req.destroy();
    handleFailure('Request timeout');
  });
}

function handleFailure(reason) {
  consecutiveFailures++;
  lastStatus = 'down';
  updateTrayMenu('✗ Offline');
  
  // Only notify on first failure to avoid spam
  if (consecutiveFailures === 1) {
    showNotification('Website is down!', `${CONFIG.url} is not responding.\nReason: ${reason}`);
  }
}

function showNotification(title, body) {
  if (Notification.isSupported()) {
    new Notification({
      title: title,
      body: body,
      icon: path.join(__dirname, 'icon.png')
    }).show();
  }
}

function startMonitoring() {
  // Initial check
  checkWebsite();
  
  // Set up interval
  monitorInterval = setInterval(() => {
    checkWebsite();
  }, CONFIG.checkInterval);
}

app.whenReady().then(() => {
  createWindow();
  createTrayIcon();
  startMonitoring();
});

app.on('window-all-closed', (e) => {
  e.preventDefault();
});

app.on('before-quit', () => {
  if (monitorInterval) {
    clearInterval(monitorInterval);
  }
});