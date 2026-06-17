document.addEventListener("DOMContentLoaded", () => {
    const audio = document.getElementById("audio-player");
    if (!audio) return;

    const playButton = document.getElementById("play-button");
    const playIcon = document.getElementById("play-icon");
    const pauseIcon = document.getElementById("pause-icon");

    const seekBar = document.getElementById("seek-bar");
    const seekProgress = document.getElementById("seek-progress");
    const seekContainer = document.querySelector(".seek-container");

    const currentTimeSpan = document.getElementById("current-time");
    const durationSpan = document.getElementById("duration");

    const volumeButton = document.getElementById("volume-button");
    const volumePopup = document.getElementById("volume-popup");

    const volumeSlider = document.getElementById("volume-slider");
    const volumeProgress = document.getElementById("volume-progress");
    const volumeThumb = document.getElementById("volume-thumb");

    function formatTime(seconds) {
        if (!isFinite(seconds) || seconds < 0) return "0:00";
        const mins = Math.floor(seconds / 60);
        const secs = Math.floor(seconds % 60);
        return `${mins}:${secs.toString().padStart(2, "0")}`;
    }

    let isSeeking = false;

    function setSeekFromTime(time) {
        if (!audio.duration) return;

        const rect = seekContainer.getBoundingClientRect();
        const pct = time / audio.duration;
        const px = pct * rect.width;

        seekProgress.style.width = `${px}px`;
        seekBar.value = pct * 100;

        currentTimeSpan.textContent = formatTime(time);
    }

    function updateSeek() {
        setSeekFromTime(audio.currentTime);
    }

    seekBar.addEventListener("input", (e) => {
        if (!audio.duration) return;

        isSeeking = true;

        const pct = parseFloat(e.target.value) / 100;
        const time = pct * audio.duration;
        setSeekFromTime(time);
    });

    seekBar.addEventListener("change", (e) => {
        if (!audio.duration) return;

        const pct = parseFloat(e.target.value) / 100;
        audio.currentTime = pct * audio.duration;
    });

    audio.addEventListener("seeked", () => {
        if (isSeeking) {
            isSeeking = false;
            updateSeek();
        }
    });

    audio.addEventListener("timeupdate", () => {
        if (!isSeeking) {
            updateSeek();
        }
    });

    function updateDuration() {
        durationSpan.textContent = formatTime(audio.duration);
    }

    if (audio.readyState >= 1) {
        updateDuration();
    }
    audio.addEventListener("loadedmetadata", updateDuration);

    audio.addEventListener("ended", () => {
        setSeekFromTime(0);
    });

    playButton.addEventListener("click", () => {
        if (audio.paused) {
            audio.play().catch(console.error);
        } else {
            audio.pause();
        }
    });

    audio.addEventListener("play", () => {
        playIcon.classList.add("hidden");
        pauseIcon.classList.remove("hidden");
    });

    audio.addEventListener("pause", () => {
        playIcon.classList.remove("hidden");
        pauseIcon.classList.add("hidden");
    });

    audio.addEventListener("ended", () => {
        playIcon.classList.remove("hidden");
        pauseIcon.classList.add("hidden");
        setSeekFromTime(0);
    });

    function updateVolumeUI(vol) {
        const pct = vol * 100;
        volumeProgress.style.height = `${pct}%`;
        volumeThumb.style.bottom = `calc(${pct}% - 5px)`;

        if (parseFloat(volumeSlider.value) !== vol) {
            volumeSlider.value = vol;
        }
    }

    audio.volume = 1;
    updateVolumeUI(audio.volume);

    volumeButton.addEventListener("click", (e) => {
        e.stopPropagation();
        volumePopup.classList.toggle("hidden");
    });

    volumeSlider.addEventListener("input", (e) => {
        const volume = parseFloat(e.target.value);
        audio.volume = volume;
        updateVolumeUI(volume);
    });

    document.addEventListener("click", (e) => {
        if (!volumeButton.contains(e.target) && !volumePopup.contains(e.target)) {
            volumePopup.classList.add("hidden");
        }
    });
});
