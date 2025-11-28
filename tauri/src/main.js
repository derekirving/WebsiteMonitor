const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// import { listen } from '@tauri-apps/api/event';

let greetInputEl;
let greetMsgEl;

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
}

async function checkWebsites() {

    let checkElem = document.querySelector('#checkResult');
    checkElem.textContent = "";

    try {
        checkElem.textContent = await invoke('check_websites');
    } catch (error) {
        console.error('Failed to check websites:', error);
    }
}

listen('website_check_complete', (event) => {
  let checkElem = document.querySelector('#checkResult');
  checkElem.textContent = event.payload;
});

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  document.querySelector("#greet-form").addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });

  document.querySelector('#checkSites').addEventListener("click", (e) => {
    e.preventDefault();
    checkWebsites();
  });
});
