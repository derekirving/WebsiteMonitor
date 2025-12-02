import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

let greetInputEl: HTMLInputElement | null;
let greetMsgEl: HTMLElement | null;
let checkElem: HTMLParagraphElement | null = document.querySelector('#checkResult');

// Your Azure AD configuration
const CONFIG = {
    clientId: '0f991812-74e3-4964-a9b2-5f8ab0629d26',
    tenantId: '631e0763-1533-47eb-a5cd-0457bee5944e',
    apiBaseUrl: 'https://your-api.azurewebsites.net/api'
};

let accessToken = null;

let loginBtn: HTMLButtonElement = document.getElementById('loginBtn') as HTMLButtonElement;
//const apiBtn = document.getElementById('apiBtn');
//const statusDiv = document.getElementById('status');

loginBtn.addEventListener('click', async () => {
    loginBtn.disabled = true;
    showStatus('Signing in...', false);
    
    try {
        const result: any = await invoke('login', {
            clientId: CONFIG.clientId,
            tenantId: CONFIG.tenantId
        });
        
        accessToken = result.access_token;
        showStatus('✅ Signed in successfully!', true);
        loginBtn.style.display = 'none';
        //apiBtn.style.display = 'inline-block';
        
        console.log('Token expires in:', result.expires_in, 'seconds');
    } catch (error) {
        showStatus('❌ Sign in failed: ' + error, false, true);
        loginBtn.disabled = false;
    }
});

function showStatus(message: string, success: boolean, isError = false) {
    console.log(message, success, isError);
}

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