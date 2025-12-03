import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { login as loginHelper, fetchProtected, logout as logoutHelper, setCurrentUser, currentUser, probeAccessToken } from './helpers/auth';

let greetInputEl: HTMLInputElement | null;
let greetMsgEl: HTMLElement | null;
let checkElem: HTMLParagraphElement | null = document.querySelector('#checkResult');

// Your Azure AD configuration
const CONFIG = {
    clientId: '0f991812-74e3-4964-a9b2-5f8ab0629d26',
    tenantId: '631e0763-1533-47eb-a5cd-0457bee5944e',
    apiBaseUrl: 'https://your-api.azurewebsites.net/api'
};

// accessToken is stored in Rust keyring; frontend uses getAccessToken/fetchProtected

let loginBtn: HTMLButtonElement = document.getElementById('loginBtn') as HTMLButtonElement;
let logoutBtn: HTMLButtonElement | null = document.getElementById('logoutBtn') as HTMLButtonElement | null;
let apiBtn: HTMLButtonElement | null = document.getElementById('apiBtn') as HTMLButtonElement | null;
let usernameEl: HTMLElement | null = null;
let authStatusEl: HTMLElement | null = null;
//const apiBtn = document.getElementById('apiBtn');
//const statusDiv = document.getElementById('status');

loginBtn.addEventListener('click', async () => {
    loginBtn.disabled = true;
    showStatus('Signing in...', false);
    try {
        const result: any = await loginHelper(CONFIG.clientId, CONFIG.tenantId);
        console.log('login result', result);
        // If your Rust login returns token including id_token, you may want to parse preferred_username there.
        // For now assume Rust saved the token and you can derive the user from the token in Rust.
        // Store a placeholder or actual user if returned by `login`
        if (result && result.id_token) {
            // JS helper may have already set current user from id_token
        }
        showStatus('✅ Signed in successfully!', true);
        loginBtn.style.display = 'none';
        if (logoutBtn) logoutBtn.style.display = 'inline-block';
        // update UI username/status
        const u = currentUser();
        if (usernameEl) usernameEl.textContent = u || 'anonymous';
        if (authStatusEl) authStatusEl.textContent = 'authenticated';
    } catch (error) {
        showStatus('❌ Sign in failed: ' + error, false, true);
        loginBtn.disabled = false;
    }
});

logoutBtn?.addEventListener('click', async () => {
    const u = currentUser();
    if (!u) return;
    try {
        await logoutHelper(u);
        showStatus('Logged out', true);
        if (loginBtn) loginBtn.style.display = 'inline-block';
        if (logoutBtn) logoutBtn.style.display = 'none';
        if (usernameEl) usernameEl.textContent = 'anonymous';
        if (authStatusEl) authStatusEl.textContent = 'not authenticated';
    } catch (e) {
        console.error('logout failed', e);
    }
});

apiBtn?.addEventListener('click', async () => {
    const u = currentUser();
    if (!u) {
        console.warn('no user');
        return;
    }
    try {
        const body = await fetchProtected(CONFIG.apiBaseUrl + '/protected', u, CONFIG.clientId, CONFIG.tenantId);
        console.log('protected response', body);
    } catch (e) {
        console.error('protected call failed', e);
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
    usernameEl = document.querySelector('#username');
    authStatusEl = document.querySelector('#auth-status');
    // On startup, probe whether we have a user and valid token
    (async () => {
        const u = currentUser();
        if (u) {
            // try to probe access token; this will refresh if needed
            const ok = await probeAccessToken(u, CONFIG.clientId, CONFIG.tenantId);
            if (usernameEl) usernameEl.textContent = u;
            if (authStatusEl) authStatusEl.textContent = ok ? 'authenticated' : 'needs login';
            if (ok) {
                if (loginBtn) loginBtn.style.display = 'none';
                if (logoutBtn) logoutBtn.style.display = 'inline-block';
            } else {
                if (loginBtn) loginBtn.style.display = 'inline-block';
                if (logoutBtn) logoutBtn.style.display = 'none';
            }
        } else {
            if (usernameEl) usernameEl.textContent = 'anonymous';
            if (authStatusEl) authStatusEl.textContent = 'not authenticated';
            if (loginBtn) loginBtn.style.display = 'inline-block';
            if (logoutBtn) logoutBtn.style.display = 'none';
        }
    })();
    document.querySelector("#greet-form")?.addEventListener("submit", (e) => {
        e.preventDefault();
        greet();
    });

    document.querySelector('#checkSites')?.addEventListener("click", (e) => {
        e.preventDefault();
        checkWebsites();
    });
});