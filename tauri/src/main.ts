import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

let greetInputEl: HTMLInputElement | null;
let greetMsgEl: HTMLElement | null;
let checkElem: HTMLParagraphElement | null = document.querySelector('#checkResult');

async function greet() {
    if (greetMsgEl && greetInputEl) {
        // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
        greetMsgEl.textContent = await invoke("greet", {
            name: greetInputEl.value,
        });
    }
}

async function checkWebsites() {

    if (checkElem) {
        checkElem.textContent = "";

        try {
            checkElem.textContent = await invoke('check_websites');
        } catch (error) {
            console.error('Failed to check websites:', error);
        }
    }
}

listen('website_check_complete', (event: any) => {
    if (checkElem) {
        checkElem.textContent = event.payload;
    }
});

window.addEventListener("DOMContentLoaded", () => {
    greetInputEl = document.querySelector("#greet-input");
    greetMsgEl = document.querySelector("#greet-msg");
    document.querySelector("#greet-form")?.addEventListener("submit", (e) => {
        e.preventDefault();
        greet();
    });

    document.querySelector('#checkSites')?.addEventListener("click", (e) => {
        e.preventDefault();
        checkWebsites();
    });
});