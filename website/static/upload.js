document.addEventListener("DOMContentLoaded", () => {
    const dropZone = document.getElementById("drop-zone");
    const fileInput = document.getElementById("file-input");
    const statusText = document.getElementById("status-text");

    const uploadingItem = document.getElementById("uploading-item");
    const uploadingName = document.getElementById("uploading-name");
    const uploadingSize = document.getElementById("uploading-size");

    const uploadsList = document.getElementById("uploads-list");

    const MAX_SIZE = 200 * 1024 * 1024;
    const ALLOWED = ["mp3", "wav", "flac", "m4a"];

    const sessionUploads = [];

    dropZone.addEventListener("click", () => fileInput.click());

    ["dragenter", "dragover"].forEach(ev => {
        dropZone.addEventListener(ev, e => {
            e.preventDefault();
            dropZone.classList.add("dragover");
        });
    });

    ["dragleave", "drop"].forEach(ev => {
        dropZone.addEventListener(ev, e => {
            e.preventDefault();
            dropZone.classList.remove("dragover");
        });
    });

    dropZone.addEventListener("drop", e => {
        const files = e.dataTransfer.files;
        if (files.length) handleFile(files[0]);
    });

    fileInput.addEventListener("change", e => {
        if (e.target.files.length) handleFile(e.target.files[0]);
        fileInput.value = "";
    });

    function formatBytes(bytes) {
        if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
        return (bytes / (1024 * 1024)).toFixed(1) + " MB";
    }

    function setStatus(message) {
        statusText.textContent = message;
        statusText.classList.add("visible");
    }

    function clearStatus() {
        statusText.textContent = "";
        statusText.classList.remove("visible");
    }

    function showUploading(file) {
        uploadingName.textContent = file.name;
        uploadingSize.textContent = formatBytes(file.size);
        uploadingItem.classList.add("visible");
    }

    function hideUploading() {
        uploadingItem.classList.remove("visible");
        uploadingName.textContent = "";
        uploadingSize.textContent = "";
    }

    function renderUploads() {
        uploadsList.innerHTML = sessionUploads.map(item => `
            <a class="upload-item" href="/f/${item.upload_id}" target="_blank" rel="noopener noreferrer">
                <span class="upload-item-link">${escapeHtml(item.name)}</span>
                <span class="upload-item-meta">${item.sizeText}</span>
            </a>
        `).join("");
    }

    function escapeHtml(str) {
        return str
            .replaceAll("&", "&amp;")
            .replaceAll("<", "&lt;")
            .replaceAll(">", "&gt;")
            .replaceAll('"', "&quot;")
            .replaceAll("'", "&#039;");
    }

    function handleFile(file) {
        const ext = file.name.split(".").pop().toLowerCase();
        clearStatus();

        if (!ALLOWED.includes(ext)) {
            setStatus("Invalid format. Please use mp3, wav, flac, or m4a.");
            return;
        }

        if (file.size > MAX_SIZE) {
            setStatus("File exceeds the 200 MB limit.");
            return;
        }

        uploadFile(file);
    }

    function uploadFile(file) {
        showUploading(file);
        setStatus("Uploading...");

        const formData = new FormData();
        formData.append("file", file);

        const xhr = new XMLHttpRequest();
        xhr.open("POST", "/api/uploads", true);

        xhr.onload = () => {
            hideUploading();

            if (xhr.status < 200 || xhr.status >= 300) {
                setStatus("Upload failed. Please try again.");
                return;
            }

            try {
                const data = JSON.parse(xhr.responseText);
                if (!data.upload_id) {
                    setStatus("Upload failed. Please try again.");
                    return;
                }

                sessionUploads.unshift({
                    upload_id: data.upload_id,
                    name: file.name,
                    sizeText: formatBytes(file.size),
                });

                renderUploads();
                clearStatus();
            } catch {
                setStatus("Upload failed. Please try again.");
            }
        };

        xhr.onerror = () => {
            hideUploading();
            setStatus("Network error. Please try again.");
        };

        xhr.send(formData);
    }
});